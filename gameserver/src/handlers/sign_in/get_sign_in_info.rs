use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::sign_in;
use sonettobuf::{CmdId, GetSignInInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_sign_in_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (info, sign_in_days, addup_bonus, month_card_days, month_card_history, birthday_heroes) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        sign_in::get_sign_in_info(&conn.state.db, player_id).await?
    };

    let reply = GetSignInInfoReply {
        has_sign_in_days: sign_in_days,
        addup_sign_in_day: Some(info.addup_sign_in_day),
        has_get_addup_bonus: addup_bonus,
        open_function_time: Some(info.open_function_time),
        has_month_card_days: month_card_days,
        month_card_history: month_card_history.into_iter().map(Into::into).collect(),
        birthday_hero_ids: birthday_heroes,
        reward_mark: Some(info.reward_mark),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetSignInInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}
