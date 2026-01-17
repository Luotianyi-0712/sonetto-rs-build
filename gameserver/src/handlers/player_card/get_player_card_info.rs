use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::player_card;
use sonettobuf::{CmdId, GetPlayerCardInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_player_card_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let card_info = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        player_card::get_player_card_info(&conn.state.db, player_id).await?
    };

    let reply = GetPlayerCardInfoReply {
        player_card_info: Some(card_info.into()),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetPlayerCardInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
