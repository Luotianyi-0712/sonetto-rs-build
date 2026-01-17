use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, GetBuyPowerInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_buy_power_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let data = GetBuyPowerInfoReply {
        can_buy_count: Some(8),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetBuyPowerInfoCmd, data, 0, req.up_tag)
        .await?;
    Ok(())
}
