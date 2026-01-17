use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::util::push::on_finish_story_notify;
use database::db::game::stories;
use prost::Message;
use sonettobuf::{CmdId, GetStoryReply, UpdateStoryReply, UpdateStoryRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_story(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (finished, processing) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        let finished = stories::get_finished_stories(&conn.state.db, player_id).await?;
        let processing = stories::get_processing_stories(&conn.state.db, player_id).await?;

        (finished, processing)
    };

    let reply = GetStoryReply {
        finish_list: finished,
        processing_list: processing.into_iter().map(Into::into).collect(),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetStoryCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_update_story(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = UpdateStoryRequest::decode(&req.data[..])?;
    let story_id = request.story_id.unwrap_or_default();
    let should_notify = request.step_id == Some(-1);
    let step_id = request.step_id.unwrap_or_default();
    let favor = request.favor.unwrap_or_default();

    let user_id = ctx.lock().await.player_id.ok_or(AppError::NotLoggedIn)?;

    tracing::info!("Received update story request: {:?}", request);

    if !should_notify {
        let pool = {
            let conn = ctx.lock().await;
            conn.state.db.clone()
        };

        let existing_step: Option<i32> = sqlx::query_scalar(
            "SELECT step_id FROM user_processing_stories WHERE user_id = ? AND story_id = ?",
        )
        .bind(user_id)
        .bind(story_id)
        .fetch_optional(&pool)
        .await
        .map_err(AppError::Database)?;

        // Only update if step_id is different or record doesn't exist
        if existing_step != Some(step_id) {
            stories::update_processing_story(&pool, user_id, story_id, step_id, favor).await?;
        }
    }

    let reply = UpdateStoryReply {};
    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::UpdateStoryCmd, reply, 0, req.up_tag)
            .await?;
    }

    if should_notify {
        on_finish_story_notify(ctx.clone(), user_id, story_id).await?;
    }

    Ok(())
}
