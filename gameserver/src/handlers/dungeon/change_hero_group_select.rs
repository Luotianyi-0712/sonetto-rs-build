use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use prost::Message;
use sonettobuf::{ChangeHeroGroupSelectReply, ChangeHeroGroupSelectRequest, CmdId};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_change_hero_group_select(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = ChangeHeroGroupSelectRequest::decode(&req.data[..])?;

    let id = request.id.ok_or(AppError::InvalidRequest)?;
    let current_select = request.current_select.ok_or(AppError::InvalidRequest)?;

    tracing::info!("Changing {} to {}", id, current_select);

    let data = ChangeHeroGroupSelectReply {
        id: Some(id),
        current_select: Some(current_select),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::ChangeHeroGroupSelectCmd, data, 0, req.up_tag)
        .await?;
    Ok(())
}
