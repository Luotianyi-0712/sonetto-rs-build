use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use codegen::configs;
use database::db::game::player_infos;
use sonettobuf::{CmdId, GetPlayerInfoReply, OpenInfo};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_player_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let player_info_data = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        player_infos::get_player_info_data(&conn.state.db, player_id)
            .await?
            .ok_or_else(|| AppError::Custom("Player info not found".to_string()))?
    };

    let game_data = configs::get();
    let openinfos: Vec<OpenInfo> = game_data
        .open
        .iter()
        .map(|open| OpenInfo {
            id: open.id,
            is_open: open.id != 280001 && open.id != 220104, // Both IDs should be closed
        })
        .collect();

    let reply = GetPlayerInfoReply {
        player_info: Some(player_info_data.into()),
        openinfos,
        can_rename: Some(true),
        main_thumbnail: Some(false),
        ext_rename: Some(0),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetPlayerInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
