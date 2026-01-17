use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::block_packages;
use sonettobuf::{CmdId, GetRoomInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_room_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (block_infos, building_infos, block_packages, road_infos, is_reset) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        let blocks = block_packages::get_blocks(pool, player_id).await?;
        let buildings = block_packages::get_buildings(pool, player_id).await?;
        let packages = block_packages::get_block_packages(pool, player_id).await?;
        let roads = block_packages::get_roads(pool, player_id).await?;
        let is_reset = block_packages::get_room_reset_state(pool, player_id).await?;

        (blocks, buildings, packages, roads, is_reset)
    };

    let reply = GetRoomInfoReply {
        infos: block_infos.into_iter().map(Into::into).collect(),
        is_reset: Some(is_reset),
        building_infos: building_infos.into_iter().map(Into::into).collect(),
        block_packages: block_packages.into_iter().map(Into::into).collect(),
        road_infos: road_infos.into_iter().map(Into::into).collect(),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetRoomInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
