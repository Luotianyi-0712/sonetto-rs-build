use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::send_reply;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, GetTaskInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_task_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetTaskInfoCmd,
        GetTaskInfoReply,
        "task/task_info.json"
    );
    Ok(())
}
