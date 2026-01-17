use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use prost::Message;
use sonettobuf::{CmdId, HeroRedDotReadReply, HeroRedDotReadRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_red_dot_read(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = HeroRedDotReadRequest::decode(&req.data[..])?;

    tracing::info!("Received HeroRedDotReadRequest: {:?}", request);

    let data = HeroRedDotReadReply {
        hero_id: Some(request.hero_id.unwrap_or(3080)),
        red_dot: Some(6),
    };

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::HeroRedDotReadCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
