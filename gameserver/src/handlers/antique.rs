use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::antiques;
use sonettobuf::{CmdId, GetAntiqueInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_antique_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let antique_list = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        antiques::get_user_antiques(&conn.state.db, player_id).await?
    };

    let reply = GetAntiqueInfoReply {
        antiques: antique_list.into_iter().map(Into::into).collect(),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetAntiqueInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
