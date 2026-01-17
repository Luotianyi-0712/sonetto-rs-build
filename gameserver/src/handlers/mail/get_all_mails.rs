use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use prost::Message;
use sonettobuf::{CmdId, GetAllMailsReply, GetAllMailsRequest, Mail};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_all_mails(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = GetAllMailsRequest::decode(&req.data[..])?;
    tracing::info!("Received GetAllMailsRequest: {:?}", request);

    let mails = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;
        let now = common::time::ServerTime::now_ms();

        sqlx::query(
            "INSERT INTO user_mail_history
             (user_id, mail_incr_id, mail_id, attachment, action, action_time, state_at_action)
             SELECT user_id, incr_id, mail_id, attachment, 'expired', ?, state
             FROM user_mails
             WHERE user_id = ? AND expire_time > 0 AND expire_time < ?",
        )
        .bind(now)
        .bind(player_id)
        .bind(now)
        .execute(pool)
        .await?;

        let deleted = sqlx::query(
            "DELETE FROM user_mails WHERE user_id = ? AND expire_time > 0 AND expire_time < ?",
        )
        .bind(player_id)
        .bind(now)
        .execute(pool)
        .await?;

        if deleted.rows_affected() > 0 {
            tracing::info!(
                "Deleted {} expired mails for user {}",
                deleted.rows_affected(),
                player_id
            );
        }

        let mail_records: Vec<(
            i64,
            i32,
            String,
            String,
            i32,
            i64,
            String,
            String,
            String,
            String,
            i64,
            i32,
            String,
            String,
        )> = sqlx::query_as(
            "SELECT incr_id, mail_id, params, attachment, state, create_time,
                    sender, title, content, copy, expire_time, sender_type,
                    jump_title, jump
             FROM user_mails
             WHERE user_id = ?
             ORDER BY create_time DESC",
        )
        .bind(player_id)
        .fetch_all(pool)
        .await?;

        let mails: Vec<Mail> = mail_records
            .into_iter()
            .map(
                |(
                    incr_id,
                    mail_id,
                    params,
                    attachment,
                    state,
                    create_time,
                    sender,
                    title,
                    content,
                    copy,
                    expire_time,
                    sender_type,
                    jump_title,
                    jump,
                )| Mail {
                    incr_id: Some(incr_id as u64),
                    mail_id: Some(mail_id as u32),
                    params: Some(params),
                    attachment: Some(attachment),
                    state: Some(state as u32),
                    create_time: Some(create_time as u64),
                    sender: Some(sender),
                    title: Some(title),
                    content: Some(content),
                    copy: Some(copy),
                    expire_time: Some(expire_time as u64),
                    sender_type: Some(sender_type),
                    jump_title: Some(jump_title),
                    jump: Some(jump),
                },
            )
            .collect();

        tracing::info!("User {} has {} active mails", player_id, mails.len());

        mails
    };

    let data = GetAllMailsReply { mails };

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::GetAllMailsCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
