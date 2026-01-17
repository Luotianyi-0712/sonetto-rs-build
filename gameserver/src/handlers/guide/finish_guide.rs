use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use prost::Message;
use sonettobuf::{CmdId, FinishGuideReply, FinishGuideRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_finish_guide(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = FinishGuideRequest::decode(&req.data[..])?;

    tracing::info!("Received FinishGuideRequest: {:?}", request);

    let reply = FinishGuideReply {};

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::FinishGuideCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
