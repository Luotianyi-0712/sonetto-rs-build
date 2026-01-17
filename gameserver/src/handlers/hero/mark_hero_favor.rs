use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::heroes;
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, MarkHeroFavorReply, MarkHeroFavorRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_mark_hero_favor(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = MarkHeroFavorRequest::decode(&req.data[..])?;
    tracing::info!("Received MarkHeroFavorRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let is_favor = request.is_favor.ok_or(AppError::InvalidRequest)?;

    let updated_hero = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        // Get hero
        let mut hero = heroes::get_hero_by_hero_id(pool, player_id, hero_id).await?;

        // Set favor status
        if hero.record.is_favor != is_favor {
            sqlx::query("UPDATE heroes SET is_favor = ? WHERE uid = ?")
                .bind(is_favor)
                .bind(hero.record.uid)
                .execute(pool)
                .await?;

            hero.record.is_favor = is_favor;
        }

        tracing::info!(
            "User {} set hero {} favorite status to {}",
            player_id,
            hero_id,
            is_favor
        );
        hero
    };

    // Send main reply
    let data = MarkHeroFavorReply {
        hero_id: Some(hero_id),
        is_favor: Some(is_favor),
    };

    {
        let mut conn = ctx.lock().await;

        // Send hero update push so client refreshes the UI
        let hero_proto: sonettobuf::HeroInfo = updated_hero.into();
        let push = HeroUpdatePush {
            hero_updates: vec![hero_proto],
        };

        conn.notify(CmdId::HeroHeroUpdatePushCmd, push).await?;

        tracing::info!(
            "Sent HeroUpdatePush for hero {} with favor {}",
            hero_id,
            is_favor
        );

        conn.send_reply(CmdId::MarkHeroFavorCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
