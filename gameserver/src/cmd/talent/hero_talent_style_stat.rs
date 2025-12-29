use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::heroes;
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, TalentStyleReadReply, TalentStyleReadRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_talent_style_stat(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = TalentStyleReadRequest::decode(&req.data[..])?;
    tracing::info!("Received TalentStyleReadRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let user_id = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &ctx_guard.state.db;

        let hero = heroes::get_hero_by_hero_id(pool, player_id, hero_id).await?;

        sqlx::query("UPDATE heroes SET talent_style_red = 0 WHERE uid = ? AND user_id = ?")
            .bind(hero.record.uid)
            .bind(player_id)
            .execute(pool)
            .await?;

        tracing::info!(
            "User {} marked talent style as read for hero {}",
            player_id,
            hero_id
        );

        player_id
    };

    let data = TalentStyleReadReply {
        hero_id: Some(hero_id),
    };

    {
        let mut ctx_guard = ctx.lock().await;

        let updated_hero =
            heroes::get_hero_by_hero_id(&ctx_guard.state.db, user_id, hero_id).await?;
        ctx_guard
            .send_push(
                CmdId::HeroHeroUpdatePushCmd,
                HeroUpdatePush {
                    hero_updates: vec![updated_hero.into()],
                },
            )
            .await?;

        ctx_guard
            .send_reply(CmdId::HeroTalentStyleStatCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
