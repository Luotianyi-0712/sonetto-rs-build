use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::equipment;
use sonettobuf::{CmdId, GetEquipInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_equip_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let equipment_list = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        equipment::get_user_equipment(&conn.state.db, player_id).await?
    };

    let reply = GetEquipInfoReply {
        equips: equipment_list.into_iter().map(Into::into).collect(),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetEquipInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
