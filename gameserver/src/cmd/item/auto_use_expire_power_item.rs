use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use sonettobuf::{AutoUseExpirePowerItemReply, CmdId};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_auto_use_expire_power_item(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let data = AutoUseExpirePowerItemReply { used: Some(false) };

    let should_save = {
        let mut ctx_guard = ctx.lock().await;

        ctx_guard
            .send_reply(CmdId::AutoUseExpirePowerItemCmd, data, 0, req.up_tag)
            .await?;

        if let Some(ps) = ctx_guard.player_state.as_mut() {
            if !ps.initial_login_complete {
                tracing::info!("Completing initial login for player {}", ps.player_id);
                ps.mark_login_complete(common::time::ServerTime::now_ms());

                ps.last_state_push_sent_timestamp = None;
                ps.last_activity_push_sent_timestamp = None;

                true
            } else {
                false
            }
        } else {
            false
        }
    };

    if should_save {
        let ctx_guard = ctx.lock().await;
        ctx_guard.save_current_player_state().await?;
    }

    Ok(())
}
