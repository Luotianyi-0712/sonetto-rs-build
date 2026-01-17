use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::send_reply;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, GetPowerMakerInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_power_maker_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetPowerMakerInfoCmd,
        GetPowerMakerInfoReply,
        "power_maker/power_maker_info.json"
    );
    Ok(())
}
