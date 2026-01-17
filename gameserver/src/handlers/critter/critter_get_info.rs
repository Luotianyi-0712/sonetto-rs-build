use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::send_reply;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, CritterGetInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_critter_get_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::CritterGetInfoCmd,
        CritterGetInfoReply,
        "critter/critter_get_info.json"
    );
    Ok(())
}
