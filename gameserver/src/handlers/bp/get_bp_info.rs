use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, GetBpInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_bp_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let resp = GetBpInfoReply {
        end_time: Some(0),
        ..Default::default()
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetBpInfoCmd, resp, 0, req.up_tag)
        .await?;

    Ok(())
}
