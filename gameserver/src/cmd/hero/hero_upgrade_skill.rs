use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::heroes;
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, HeroUpgradeSkillReply, HeroUpgradeSkillRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_upgrade_skill(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = HeroUpgradeSkillRequest::decode(&req.data[..])?;
    let hero_id = request.hero_id;
    let skill_type = request.r#type; // 3 = ex_skill
    let consume = request.consume.unwrap_or(1);

    tracing::info!("Received HeroUpgradeSkillRequest: {:?}", request);

    let (updated_hero, consumed_item_id, player_id) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &ctx_guard.state.db;

        let mut hero = heroes::get_hero_by_hero_id(pool, player_id, hero_id).await?;

        if skill_type == 3 && hero.record.ex_skill_level >= 5 {
            return Err(AppError::InvalidRequest);
        }

        let game_data = data::exceldb::get();
        let character = game_data
            .character
            .iter()
            .find(|c| c.id == hero_id)
            .ok_or(AppError::InvalidRequest)?;

        // Parse duplicateItem: "1#133125#1|2#11#12"
        let dupe_item_id = character
            .duplicate_item
            .split('|')
            .next()
            .and_then(|part| {
                let segments: Vec<&str> = part.split('#').collect();
                if segments.len() >= 3 && segments[0] == "1" {
                    segments[1].parse::<u32>().ok()
                } else {
                    None
                }
            })
            .ok_or(AppError::InvalidRequest)?;

        let has_item = database::db::game::items::get_item(pool, player_id, dupe_item_id)
            .await?
            .map(|i| i.quantity >= consume)
            .unwrap_or(false);

        if !has_item {
            return Err(AppError::InsufficientItems);
        }

        database::db::game::items::remove_item_quantity(pool, player_id, dupe_item_id, consume)
            .await?;

        if skill_type == 3 {
            hero.record.ex_skill_level += consume;
            hero.record.ex_skill_level = hero.record.ex_skill_level.min(5);

            sqlx::query("UPDATE heroes SET ex_skill_level = ? WHERE user_id = ? AND hero_id = ?")
                .bind(hero.record.ex_skill_level)
                .bind(player_id)
                .bind(hero_id)
                .execute(pool)
                .await?;

            tracing::info!(
                "User {} upgraded ex_skill to level {} on hero {}",
                player_id,
                hero.record.ex_skill_level,
                hero_id
            );
        }

        (hero, dupe_item_id, player_id)
    };

    crate::utils::push::send_item_change_push(
        ctx.clone(),
        player_id,
        vec![consumed_item_id],
        vec![],
        vec![],
    )
    .await?;

    let data = HeroUpgradeSkillReply {};

    {
        let mut ctx_guard = ctx.lock().await;

        let hero_proto: sonettobuf::HeroInfo = updated_hero.into();
        let push = HeroUpdatePush {
            hero_updates: vec![hero_proto],
        };
        ctx_guard
            .send_push(CmdId::HeroHeroUpdatePushCmd, push)
            .await?;

        tracing::info!("Sent HeroUpdatePush for hero {} ex_skill upgrade", hero_id);
    }

    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(CmdId::HeroUpgradeSkillCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
