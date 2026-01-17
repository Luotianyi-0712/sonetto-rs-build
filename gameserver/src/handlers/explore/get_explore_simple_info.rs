use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::explore;
use sonettobuf::{CmdId, GetExploreSimpleInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_explore_simple_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (info, chapters, maps, unlocked_maps) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        explore::get_explore_info(&conn.state.db, player_id).await?
    };

    let reply = GetExploreSimpleInfoReply {
        last_map_id: Some(info.last_map_id),
        chapter_simple: chapters.into_iter().map(Into::into).collect(),
        map_simple: maps.into_iter().map(Into::into).collect(),
        unlock_map_ids: unlocked_maps,
        is_show_bag: Some(info.is_show_bag),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetExploreSimpleInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
