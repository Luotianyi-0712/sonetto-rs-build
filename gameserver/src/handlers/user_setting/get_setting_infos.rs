use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::send_reply;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, GetSettingInfosReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_setting_infos(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetSettingInfosCmd,
        GetSettingInfosReply,
        "user_setting/setting_infos.json"
    );
    Ok(())
}
