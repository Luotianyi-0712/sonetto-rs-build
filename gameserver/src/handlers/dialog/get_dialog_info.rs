use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::dialogs;
use sonettobuf::{CmdId, GetDialogInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_dialog_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let dialog_ids = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        dialogs::get_dialog_ids(&conn.state.db, player_id).await?
    };

    let reply = GetDialogInfoReply { dialog_ids };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetDialogInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
