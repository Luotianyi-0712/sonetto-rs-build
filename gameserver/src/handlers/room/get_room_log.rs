use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::send_reply;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, GetRoomLogReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_room_log(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetRoomLogCmd,
        GetRoomLogReply,
        "room/room_log.json"
    );
    Ok(())
}
