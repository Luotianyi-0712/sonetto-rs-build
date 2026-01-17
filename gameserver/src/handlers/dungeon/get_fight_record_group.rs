use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::dungeons::load_dungeon_record;
use prost::Message;
use sonettobuf::{CmdId, GetFightRecordGroupReply, GetFightRecordGroupRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_fight_record_group(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = GetFightRecordGroupRequest::decode(&req.data[..])?;
    let episode_id = request.episode_id.unwrap_or(0);

    tracing::info!("GetFightRecordGroup for episode {}", episode_id);

    let (pool, player_id) = {
        let conn = ctx.lock().await;
        (conn.state.db.clone(), conn.player_id)
    };

    let record = load_dungeon_record(&pool, player_id.unwrap_or(0), episode_id).await?;

    tracing::info!("Loaded record: {:?}", record.is_some());
    if let Some(ref rec) = record {
        tracing::info!(
            "Record has {} heroes, {} trial heroes",
            rec.hero_list.len(),
            rec.trial_hero_list.len()
        );
    }

    let reply = GetFightRecordGroupReply {
        fight_group: record,
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetFightRecordGroupCmd, reply, 0, req.up_tag)
        .await?;

    tracing::info!("Sent GetFightRecordGroup reply");

    Ok(())
}
