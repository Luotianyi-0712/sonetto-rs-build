use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, GetHeroGroupListReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_hero_group_list(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let group_info = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        database::db::game::hero_groups::get_current_hero_group(&conn.state.db, player_id).await?
    };

    let reply = GetHeroGroupListReply {
        group_info_list: if let Some(info) = group_info {
            vec![info.into()]
        } else {
            vec![]
        },
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetHeroGroupListCmd, reply, 0, req.up_tag)
        .await?;
    Ok(())
}
