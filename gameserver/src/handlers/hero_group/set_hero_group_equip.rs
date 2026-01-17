use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::hero_groups;
use prost::Message;
use sonettobuf::{CmdId, HeroGroupEquip, SetHeroGroupEquipReply, SetHeroGroupEquipRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_set_hero_group_equip(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = SetHeroGroupEquipRequest::decode(&req.data[..])?;
    tracing::info!("Received SetHeroGroupEquipRequest: {:?}", request);

    let group_id = request.group_id.ok_or(AppError::InvalidRequest)?;
    let equip = request.equip.ok_or(AppError::InvalidRequest)?;
    let index = equip.index.ok_or(AppError::InvalidRequest)?;
    let equip_uids = equip.equip_uid.clone();

    {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        // Update the equipment
        hero_groups::set_hero_group_equip(pool, player_id, group_id, index, equip_uids.clone())
            .await?;

        tracing::info!(
            "User {} set group {} index {} to equips: {:?}",
            player_id,
            group_id,
            index,
            equip_uids
        );
    }

    let data = SetHeroGroupEquipReply {
        group_id: Some(group_id),
        equip: Some(HeroGroupEquip {
            index: Some(index),
            equip_uid: equip_uids,
        }),
    };

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::SetHeroGroupEquipCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
