use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::weekwalk;
use sonettobuf::{CmdId, GetWeekwalkInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_weekwalk_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (info, map_infos) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        weekwalk::get_weekwalk_info(&conn.state.db, player_id).await?
    };

    let reply = GetWeekwalkInfoReply {
        info: Some(sonettobuf::WeekwalkInfo {
            time: Some(info.time),
            end_time: Some(info.end_time),
            map_info: map_infos,
            max_layer: Some(info.max_layer),
            issue_id: Some(info.issue_id),
            is_pop_deep_rule: Some(info.is_pop_deep_rule),
            is_open_deep: Some(info.is_open_deep),
            is_pop_shallow_settle: Some(info.is_pop_shallow_settle),
            is_pop_deep_settle: Some(info.is_pop_deep_settle),
            deep_progress: Some(info.deep_progress),
        }),
        time_this_week: Some(info.time_this_week),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetWeekwalkInfoCmd, reply, 0, req.up_tag)
        .await?;
    Ok(())
}
