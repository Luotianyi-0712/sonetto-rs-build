use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use prost::Message;
use sonettobuf::{CmdId, EndDungeonReply, EndDungeonRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_dungeon_end_dungeon(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = EndDungeonRequest::decode(&req.data[..])?;

    let is_abort = request.is_abort.ok_or(AppError::InvalidRequest)?;

    tracing::info!("Dungeon ended with is_abort: {}", is_abort);

    // Clear battle
    {
        let mut conn = ctx.lock().await;
        conn.active_battle = None;
    }

    let data = EndDungeonReply {};

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::DungeonEndDungeonCmd, data, 0, req.up_tag)
        .await?;
    Ok(())
}
