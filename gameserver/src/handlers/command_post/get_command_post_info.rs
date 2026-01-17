use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::command_post;
use sonettobuf::{CmdId, GetCommandPostInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_command_post_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (info, events, tasks, catch_tasks, gain_bonus) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        command_post::get_command_post_info(&conn.state.db, player_id).await?
    };

    let reply = GetCommandPostInfoReply {
        event_list: events.into_iter().map(Into::into).collect(),
        tasks: tasks.into_iter().map(Into::into).collect(),
        catch_tasks: catch_tasks.into_iter().map(Into::into).collect(),
        gain_bonus,
        paper: Some(info.paper),
        catch_num: Some(info.catch_num),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetCommandPostInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
