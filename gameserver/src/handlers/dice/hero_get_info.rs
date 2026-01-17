use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::{error::AppError, send_reply};
use sonettobuf::{CmdId, DiceHeroGetInfoReply};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_dice_hero_get_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::DiceHeroGetInfoCmd,
        DiceHeroGetInfoReply,
        "dice/dice_hero.json"
    );

    Ok(())
}
