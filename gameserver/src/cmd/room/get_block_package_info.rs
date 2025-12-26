use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::block_packages;
use sonettobuf::{CmdId, GetBlockPackageInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_block_package_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (packages, special_blocks) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;

        let packages = block_packages::get_block_packages(&ctx_guard.state.db, player_id).await?;
        let blocks = block_packages::get_special_blocks(&ctx_guard.state.db, player_id).await?;

        (packages, blocks)
    };

    let reply = GetBlockPackageInfoReply {
        block_package_ids: packages.into_iter().map(|p| p.block_package_id).collect(),
        special_blocks: special_blocks.into_iter().map(Into::into).collect(),
    };

    let mut ctx_guard = ctx.lock().await;
    ctx_guard
        .send_reply(CmdId::GetBlockPackageInfoRequsetCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
