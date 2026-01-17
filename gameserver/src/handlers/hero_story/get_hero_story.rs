use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::send_reply;
use crate::state::ConnectionContext;
use sonettobuf::{CmdId, GetHeroStoryReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_hero_story(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetHeroStoryCmd,
        GetHeroStoryReply,
        "hero_story/hero_story.json"
    );
    Ok(())
}
