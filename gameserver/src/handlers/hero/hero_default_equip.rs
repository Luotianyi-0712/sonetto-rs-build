use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::heroes;
use prost::Message;
use sonettobuf::{CmdId, HeroDefaultEquipReply, HeroDefaultEquipRequest, HeroUpdatePush};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_default_equip(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = HeroDefaultEquipRequest::decode(&req.data[..])?;
    tracing::info!("Received HeroDefaultEquipRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let equip_uid = request.default_equip_uid.ok_or(AppError::InvalidRequest)?;

    let updated_hero = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        // Get hero
        let mut hero = heroes::get_hero_by_hero_id(pool, player_id, hero_id).await?;

        // Update equipped gear
        hero.update_equipped_gear(pool, equip_uid).await?;

        tracing::info!(
            "User {} equipped gear {} on hero {}",
            player_id,
            equip_uid,
            hero_id
        );
        hero
    };

    // Send main reply
    let data = HeroDefaultEquipReply {
        hero_id: Some(hero_id),
        default_equip_uid: Some(equip_uid),
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
            "Sent HeroUpdatePush for hero {} with equip {}",
            hero_id,
            equip_uid
        );

        conn.send_reply(CmdId::HeroDefaultEquipCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
