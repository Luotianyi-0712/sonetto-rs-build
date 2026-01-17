use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::heroes::*;
use sonettobuf::{CmdId, HeroBirthdayInfo, HeroInfoListReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_info_list(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (_player_id, heroes_data, touch_count, all_skins, birthday_infos) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        let heroes = get_user_heroes(&conn.state.db, player_id)
            .await
            .map_err(|e| AppError::Custom(format!("Failed to load heroes: {}", e)))?;

        let touch_count = get_touch_count(&conn.state.db, player_id)
            .await
            .map_err(|e| AppError::Custom(format!("Failed to load touch count: {}", e)))?
            .unwrap_or(5);

        let all_skins = get_all_hero_skins(&conn.state.db, player_id)
            .await
            .map_err(|e| AppError::Custom(format!("Failed to load hero skins: {}", e)))?;

        let birthday_infos = get_birthday_info(&conn.state.db, player_id)
            .await
            .map_err(|e| AppError::Custom(format!("Failed to load birthday info: {}", e)))?;

        (player_id, heroes, touch_count, all_skins, birthday_infos)
    };

    let reply = HeroInfoListReply {
        heros: heroes_data.into_iter().map(Into::into).collect(),
        touch_count_left: Some(touch_count),
        all_hero_skin: all_skins,
        birthday_infos: birthday_infos
            .into_iter()
            .map(|(hero_id, count)| HeroBirthdayInfo {
                hero_id: Some(hero_id),
                birthday_count: Some(count),
            })
            .collect(),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::HeroInfoListCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
