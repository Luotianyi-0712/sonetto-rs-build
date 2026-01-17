use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::buildings;
use sonettobuf::{CmdId, GetBuildingInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_building_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let building_infos = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        buildings::get_user_buildings(&conn.state.db, player_id).await?
    };

    let reply = GetBuildingInfoReply {
        building_infos: building_infos.into_iter().map(Into::into).collect(),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetBuildingInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
