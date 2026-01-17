use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::send_reply;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, InstructionDungeonInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_instruction_dungeon_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::DungeonInstructionDungeonInfoCmd,
        InstructionDungeonInfoReply,
        "dungeon/instruction_dungeon_info.json"
    );
    Ok(())
}
