use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::{cmd::friend::util::handle_command, error::AppError};
use database::db::game::player_infos::get_player_info_data;
use prost::Message;
use sonettobuf::{ChatMsg, ChatMsgPush, CmdId, SendMsgReply, SendMsgRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_send_msg(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = SendMsgRequest::decode(&req.data[..])?;
    tracing::info!("Received SendMsgRequest: {:?}", request);

    let input = request.content.clone().unwrap_or_default();

    let (player_id, username, portrait, level) = {
        let ctx_guard = ctx.lock().await;
        let pool = ctx_guard.state.db.clone();
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let player_data = get_player_info_data(&pool, player_id)
            .await?
            .ok_or(AppError::InvalidRequest)?;
        (
            player_id as u64,
            player_data.user_info.username.clone(),
            player_data.player_info.portrait as u32,
            player_data.user_info.level as u32,
        )
    };

    let now = common::time::ServerTime::now_ms() as u64;
    let mut messages = Vec::new();

    // game expects the message to be in a specific order or else it fails
    // also since this message is for bot usage only we only make it exists in the current session
    // ig resets every time you log in
    let (player_msg_id, bot_msg_id) = {
        let mut ctx_guard = ctx.lock().await;
        let base = ctx_guard.bot_msg_counter;
        ctx_guard.bot_msg_counter += 2;
        (base, base + 1)
    };

    messages.push(ChatMsg {
        msg_id: Some(player_msg_id),
        sender_id: Some(player_id),
        channel_type: request.channel_type.clone(),
        sender_name: Some(username),
        portrait: Some(portrait),
        content: request.content.clone(),
        send_time: Some(now),
        level: Some(level),
        recipient_id: Some(1337),
        msg_type: request.msg_type.clone(),
        ext_data: request.ext_data.clone(),
    });

    if let Some(bot_text) = handle_command(ctx.clone(), &input).await {
        messages.push(ChatMsg {
            msg_id: Some(bot_msg_id),
            sender_id: Some(1337),
            channel_type: request.channel_type.clone(),
            sender_name: Some("Sonetto Bot".to_string()),
            portrait: Some(171805),
            content: Some(bot_text),
            send_time: Some(now),
            level: Some(80),
            recipient_id: Some(player_id),
            msg_type: Some(0),
            ext_data: Some(String::new()),
        });
    }

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_push(CmdId::ChatMsgPushCmd, ChatMsgPush { msg: messages })
            .await?;
    }

    let reply = SendMsgReply {
        channel_type: request.channel_type,
        message: None,
        content: request.content,
        msg_type: request.msg_type,
        ext_data: request.ext_data,
    };

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(CmdId::SendMsgCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
