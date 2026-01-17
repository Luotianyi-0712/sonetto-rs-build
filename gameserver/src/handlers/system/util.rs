use crate::error::{AppError, PacketError};
use crate::state::ConnectionContext;
use byteorder::{BE, ByteOrder};
use sonettobuf::CmdId;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct LoginRequest {
    pub account_id: String,
    pub token: String,
}

pub fn parse_login_request(data: &[u8]) -> Result<LoginRequest, AppError> {
    if data.len() < 2 {
        return Err(AppError::Packet(PacketError::Custom(
            "LoginRequest too short".into(),
        )));
    }

    let account_id_len = BE::read_u16(&data[0..2]) as usize;

    tracing::debug!(
        "Login packet - Length prefix: {}, Total packet size: {}",
        account_id_len,
        data.len()
    );
    tracing::debug!("Full packet hex: {:02X?}", data);

    if data.len() < 2 + account_id_len {
        return Err(AppError::Packet(PacketError::Custom(
            "LoginRequest length mismatch".into(),
        )));
    }

    // Parse the account_id part
    let account_str = std::str::from_utf8(&data[2..2 + account_id_len])?;
    tracing::debug!("Account string: '{}'", account_str);

    // Check if there's more data after account_id (the token might be separate)
    let remaining_data = &data[2 + account_id_len..];
    tracing::debug!(
        "Remaining data after account_id: {} bytes, hex: {:02X?}",
        remaining_data.len(),
        remaining_data
    );

    // Try to parse token from remaining data
    let token = if remaining_data.len() >= 2 {
        let token_len = BE::read_u16(&remaining_data[0..2]) as usize;
        if remaining_data.len() >= 2 + token_len {
            let token_str = std::str::from_utf8(&remaining_data[2..2 + token_len])?;
            tracing::debug!("Found token in packet: '{}'", token_str);
            token_str.to_string()
        } else {
            tracing::warn!("Token length mismatch");
            String::new()
        }
    } else if account_str.contains('#') || account_str.contains('$') {
        // Token might be in the same string
        let separator = if account_str.contains('#') { '#' } else { '$' };
        let parts: Vec<&str> = account_str.splitn(2, separator).collect();
        if parts.len() == 2 {
            tracing::debug!("Token found in account_str: '{}'", parts[1]);
            parts[1].to_string()
        } else {
            String::new()
        }
    } else {
        tracing::warn!("No token found in packet");
        String::new()
    };

    let account_id = if account_str.contains('#') || account_str.contains('$') {
        let separator = if account_str.contains('#') { '#' } else { '$' };
        account_str.split(separator).next().unwrap_or(account_str)
    } else {
        account_str
    }
    .to_string();

    Ok(LoginRequest { account_id, token })
}

pub fn extract_user_id(account_id: &str) -> Result<i64, AppError> {
    // Format is: channelId_userId
    // We want the part after the underscore
    account_id
        .split('_')
        .nth(1) // Get second part (after first underscore)
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| AppError::Custom(format!("Invalid account_id format: {}", account_id)))
}

pub fn build_login_reply(user_id: i64) -> Vec<u8> {
    let mut payload = Vec::new();

    // Reason string (custom binary format: u16 length + string bytes)
    let reason = "OK";
    let reason_bytes = reason.as_bytes();
    let reason_len = reason_bytes.len() as u16;

    payload.extend_from_slice(&reason_len.to_be_bytes());
    payload.extend_from_slice(reason_bytes);

    // user_id as i64 (ReadLong expects signed)
    payload.extend_from_slice(&user_id.to_be_bytes());

    payload
}

pub fn build_login_error(reason: &str) -> Vec<u8> {
    let mut payload = Vec::new();

    let reason_bytes = reason.as_bytes();
    let reason_len = reason_bytes.len() as u16;

    payload.extend_from_slice(&reason_len.to_be_bytes());
    payload.extend_from_slice(reason_bytes);

    // user_id = 0 for failed login
    payload.extend_from_slice(&0i64.to_be_bytes());

    payload
}

/// Load critters from database and send push
pub async fn send_critter_push(
    ctx: Arc<Mutex<ConnectionContext>>,
    user_id: i64,
) -> Result<(), AppError> {
    let critters = {
        let conn = ctx.lock().await;
        database::db::game::critters::get_player_critters(&conn.state.db, user_id)
            .await
            .unwrap_or_default()
    };

    let mut conn = ctx.lock().await;
    let push = sonettobuf::CritterInfoPush {
        critter_infos: critters.into_iter().map(Into::into).collect(),
    };
    conn.notify(CmdId::CritterInfoPushCmd, push).await?;

    Ok(())
}

pub async fn login_error(
    ctx: &Arc<Mutex<ConnectionContext>>,
    msg: &str,
    up_tag: u8,
) -> Result<(), AppError> {
    let mut ctx = ctx.lock().await;
    let payload = build_login_error(msg);
    ctx.send_raw_reply_fixed(CmdId::LoginRequestCmd, payload, 1, up_tag)
        .await?;
    Err(AppError::Custom(msg.to_string()))
}
