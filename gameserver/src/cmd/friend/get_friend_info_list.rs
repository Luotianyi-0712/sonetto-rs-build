use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::{cmd::friend::send_bot_welcome, error::AppError};
use sonettobuf::{CmdId, FriendInfo, GetFriendInfoListReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_friend_info_list(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let player_id = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;

        player_id as u64
    };

    let friends = vec![FriendInfo {
        user_id: Some(1337),
        level: Some(80),
        time: None, //setting a time actually makes you appear offline
        name: Some("Sonetto Bot".to_string()),
        portrait: Some(171805),
        desc: Some("".to_string()),
        infos: vec![], //this is where you friends hero profile would appear but we don't need that for a bot
        bg: None, // ig this is the background 
    }];

    let reply = GetFriendInfoListReply { info: friends };

    {
        let mut ctx_guard = ctx.lock().await;
        if !ctx_guard.bot_welcome_sent {
            let id = ctx_guard.bot_msg_counter;
            ctx_guard.bot_msg_counter += 1;
            send_bot_welcome(&mut ctx_guard, player_id, id).await?;
        }
    }

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(CmdId::GetFriendInfoListCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
