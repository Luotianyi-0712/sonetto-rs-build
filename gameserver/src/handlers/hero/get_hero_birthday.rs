use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use prost::Message;
use sonettobuf::{CmdId, GetHeroBirthdayReply, GetHeroBirthdayRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_hero_birthday(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = GetHeroBirthdayRequest::decode(&req.data[..])?;

    let data = GetHeroBirthdayReply {
        hero_id: Some(request.hero_id.unwrap_or(3080)),
    };

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::GetHeroBirthdayCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
