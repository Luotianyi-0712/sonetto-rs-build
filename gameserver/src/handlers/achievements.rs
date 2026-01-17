use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::achievements;
use sonettobuf::{CmdId, GetAchievementInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_achievement_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let achievement_infos = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        achievements::get_achievements(&conn.state.db, player_id).await?
    };

    let reply = GetAchievementInfoReply {
        infos: achievement_infos.into_iter().map(Into::into).collect(),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetAchievementInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
