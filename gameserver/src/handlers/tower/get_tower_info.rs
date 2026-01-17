use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::tower;
use sonettobuf::{CmdId, GetTowerInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_tower_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (info, tower_opens, towers, assist_bosses) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        tower::get_tower_info(&conn.state.db, player_id).await?
    };

    let reply = GetTowerInfoReply {
        tower_opens: tower_opens.into_iter().map(Into::into).collect(),
        towers,
        assist_bosses: assist_bosses.into_iter().map(Into::into).collect(),
        mop_up_times: Some(info.mop_up_times),
        trial_hero_season: Some(info.trial_hero_season),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetTowerInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
