use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use sonettobuf::{ChatMsgPush, CmdId, DeleteOfflineMsgReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_delete_offline_msg(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let reply = DeleteOfflineMsgReply {};

    // Honestly not sure if this is needed here but leave it for now
    let push = ChatMsgPush { msg: vec![] };

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard.send_push(CmdId::ChatMsgPushCmd, push).await?;
    }

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(CmdId::DeleteOfflineMsgCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
