use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::weekwalk_v2;
use sonettobuf::{CmdId, WeekwalkVer2GetInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_weekwalk_ver2_get_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let info = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        weekwalk_v2::get_weekwalk_v2_info(&conn.state.db, player_id).await?
    };

    let reply = WeekwalkVer2GetInfoReply { info: Some(info) };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::WeekwalkVer2GetInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
