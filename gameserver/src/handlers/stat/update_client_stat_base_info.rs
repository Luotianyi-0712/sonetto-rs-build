use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, UpdateClientStatBaseInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_update_client_stat_base_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let data = UpdateClientStatBaseInfoReply {
        account_bind_bonus: Some(0),
    };

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::UpdateClientStatBaseInfoCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
