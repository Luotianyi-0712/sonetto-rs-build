use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::send_reply;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, GetRoomPlanInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_room_plan_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetRoomPlanInfoCmd,
        GetRoomPlanInfoReply,
        "room/room_plan_info.json"
    );
    Ok(())
}
