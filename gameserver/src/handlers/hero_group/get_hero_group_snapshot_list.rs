use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::hero_group_snapshots;
use prost::Message;
use sonettobuf::{CmdId, GetHeroGroupSnapshotListReply, GetHeroGroupSnapshotListRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_hero_group_snapshot_list(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let req_msg = GetHeroGroupSnapshotListRequest::decode(&req.data[..])?;

    tracing::info!("Received GetHeroGroupSnapshotListRequest: {:?}", req_msg);

    let snapshots = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        let snapshot_id = req_msg.snapshot_id.unwrap_or(0);

        if snapshot_id == 0 {
            // 0 means "get ALL snapshots"
            hero_group_snapshots::get_hero_group_snapshots(&conn.state.db, player_id).await?
        } else {
            // Get specific snapshot
            let snapshot = hero_group_snapshots::get_hero_group_snapshot(
                &conn.state.db,
                player_id,
                snapshot_id,
            )
            .await?;

            if let Some(s) = snapshot {
                vec![s]
            } else {
                vec![]
            }
        }
    };

    tracing::info!("Returning {} snapshot(s)", snapshots.len());

    let reply = GetHeroGroupSnapshotListReply {
        hero_group_snapshots: snapshots.into_iter().map(Into::into).collect(),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetHeroGroupSnapshotListCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
