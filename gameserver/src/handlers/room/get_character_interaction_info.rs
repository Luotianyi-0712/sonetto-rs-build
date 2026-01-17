use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::character_interactions;
use sonettobuf::{CmdId, GetCharacterInteractionInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_character_interaction_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (infos, count) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        let infos =
            character_interactions::get_character_interactions(&conn.state.db, player_id).await?;
        let count =
            character_interactions::get_interaction_count(&conn.state.db, player_id).await?;

        (infos, count)
    };

    let reply = GetCharacterInteractionInfoReply {
        infos: infos.into_iter().map(Into::into).collect(),
        interaction_count: Some(count),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetCharacterInteractionInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
