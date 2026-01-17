use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::user_stats;
use sonettobuf::CmdId;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_client_stat_base_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let stats = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        user_stats::get_user_stats(&conn.state.db, player_id)
            .await?
            .ok_or_else(|| AppError::Custom("User stats not found".to_string()))?
    };

    {
        let mut conn = ctx.lock().await;
        conn.notify(
            CmdId::StatInfoPushCmd,
            <sonettobuf::StatInfoPush>::from(stats),
        )
        .await?;
    }

    {
        let mut conn = ctx.lock().await;
        conn.send_empty_reply(CmdId::ClientStatBaseInfoCmd, Vec::new(), 0, req.up_tag)
            .await?;
    }

    Ok(())
}
