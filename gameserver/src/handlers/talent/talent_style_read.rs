use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::heroes;
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, TalentStyleReadReply, TalentStyleReadRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_talent_style_read(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = TalentStyleReadRequest::decode(&req.data[..])?;
    tracing::info!("Received TalentStyleReadRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let user_id = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        player_id
    };

    let data = TalentStyleReadReply {
        hero_id: Some(hero_id),
    };

    {
        let mut conn = ctx.lock().await;

        let updated_hero = heroes::get_hero_by_hero_id(&conn.state.db, user_id, hero_id).await?;
        conn.notify(
            CmdId::HeroHeroUpdatePushCmd,
            HeroUpdatePush {
                hero_updates: vec![updated_hero.into()],
            },
        )
        .await?;

        conn.send_reply(CmdId::TalentStyleReadCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
