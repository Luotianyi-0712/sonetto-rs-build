use crate::network::packet::ClientPacket;
use crate::send_reply;
use crate::{error::AppError, state::ConnectionContext};
use sonettobuf::{CmdId, GetManufactureInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_manufacture_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetManufactureInfoCmd,
        GetManufactureInfoReply,
        "manufacture/manufacture_info.json"
    );

    Ok(())
}
