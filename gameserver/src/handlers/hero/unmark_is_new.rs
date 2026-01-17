use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::heroes;
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, UnMarkIsNewReply, UnMarkIsNewRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_unmark_is_new(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = UnMarkIsNewRequest::decode(&req.data[..])?;
    tracing::info!("Received UnMarkIsNewRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let updated_hero = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        let mut hero = heroes::get_hero_by_hero_id(pool, player_id, hero_id).await?;

        if hero.record.is_new {
            sqlx::query("UPDATE heroes SET is_new = ? WHERE uid = ?")
                .bind(false)
                .bind(hero.record.uid)
                .execute(pool)
                .await?;

            hero.record.is_new = false;

            tracing::info!("User {} unmarked hero {} as new", player_id, hero_id);
        }

        hero
    };

    let data = UnMarkIsNewReply {
        hero_id: Some(hero_id),
    };

    {
        let mut conn = ctx.lock().await;
        let hero_proto: sonettobuf::HeroInfo = updated_hero.into();
        let push = HeroUpdatePush {
            hero_updates: vec![hero_proto],
        };
        conn.notify(CmdId::HeroHeroUpdatePushCmd, push).await?;

        conn.send_reply(CmdId::UnMarkIsNewCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
