use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::{cmd::item::apply_insight_item, error::AppError};
use prost::Message;
use sonettobuf::{CmdId, UseInsightItemReply, UseInsightItemRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_use_insight_item(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = UseInsightItemRequest::decode(&req.data[..])?;
    tracing::info!("Received UseInsightItemRequest: {:?}", request);

    let uid = request.uid.ok_or(AppError::InvalidRequest)?;
    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let ctx_guard = ctx.lock().await;
        (
            ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?,
            ctx_guard.state.db.clone(),
        )
    };

    let item_id = apply_insight_item(&pool, player_id, uid, hero_id).await?;

    crate::utils::push::send_item_change_push(
        ctx.clone(),
        player_id,
        vec![],
        vec![],
        vec![item_id as u32],
    )
    .await?;

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(
                CmdId::UseInsightItemCmd,
                UseInsightItemReply {
                    hero_id: Some(hero_id),
                    uid: Some(uid),
                },
                0,
                req.up_tag,
            )
            .await?;
    }

    if let Ok(hero) =
        database::db::game::heroes::get_hero_by_hero_id(&pool, player_id, hero_id).await
    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_push(
                CmdId::HeroHeroUpdatePushCmd,
                sonettobuf::HeroUpdatePush {
                    hero_updates: vec![hero.into()],
                },
            )
            .await?;
    }

    Ok(())
}
