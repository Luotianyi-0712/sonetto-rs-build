use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::send_reply;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, GetHandbookInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_handbook_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetHandbookInfoCmd,
        GetHandbookInfoReply,
        "handbook/handbook_info.json"
    );
    Ok(())
}
