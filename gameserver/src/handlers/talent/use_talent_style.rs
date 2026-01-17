use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::heroes;
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, UseTalentStyleReply, UseTalentStyleRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_use_talent_style(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = UseTalentStyleRequest::decode(&req.data[..])?;
    tracing::info!("Received UseTalentStyleRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let template_id = request.template_id.ok_or(AppError::InvalidRequest)?;
    let style = request.style.ok_or(AppError::InvalidRequest)?;

    let user_id = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        let hero = heroes::get_hero_by_hero_id(pool, player_id, hero_id).await?;

        if style != 0 {
            let has_style: Option<i32> = sqlx::query_scalar(
                "SELECT 1 FROM hero_talent_styles WHERE hero_uid = ? AND style_id = ?",
            )
            .bind(hero.record.uid)
            .bind(style)
            .fetch_optional(pool)
            .await?;

            if has_style.is_none() {
                tracing::warn!(
                    "User {} does not own style {} for hero {}",
                    player_id,
                    style,
                    hero_id
                );
                return Err(AppError::InvalidRequest);
            }
        }

        let template_row_id: i64 = sqlx::query_scalar(
            "SELECT rowid FROM hero_talent_templates WHERE hero_uid = ? AND template_id = ?",
        )
        .bind(hero.record.uid)
        .bind(template_id)
        .fetch_one(pool)
        .await?;

        sqlx::query("UPDATE hero_talent_templates SET style = ? WHERE rowid = ?")
            .bind(style)
            .bind(template_row_id)
            .execute(pool)
            .await?;

        tracing::info!(
            "User {} applied style {} to template {} for hero {}",
            player_id,
            style,
            template_id,
            hero_id
        );

        player_id
    };

    let data = UseTalentStyleReply {
        hero_id: Some(hero_id),
        template_id: Some(template_id),
        style: Some(style),
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

        conn.send_reply(CmdId::UseTalentStyleCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
