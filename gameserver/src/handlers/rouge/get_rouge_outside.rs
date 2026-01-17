use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::send_reply;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, GetRougeOutsideInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_rouge_outside_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetRougeOutsideInfoCmd,
        GetRougeOutsideInfoReply,
        "rouge/rouge_outside_info.json"
    );
    Ok(())
}
