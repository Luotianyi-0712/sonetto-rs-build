use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::util::push;
use database::db::game::heroes;
use prost::Message;
use sonettobuf::{CmdId, HeroLevelUpReply, HeroLevelUpRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_level_up(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = HeroLevelUpRequest::decode(&req.data[..])?;
    tracing::info!("Received HeroLevelUpRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let expect_level = request.expect_level.ok_or(AppError::InvalidRequest)?;

    let (user_id, new_rank, consumed_currencies) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        let hero = heroes::get_hero_by_hero_id(pool, player_id, hero_id).await?;
        let old_level = hero.record.level;

        if expect_level <= hero.record.level {
            if expect_level == hero.record.level {
                tracing::info!("Hero {} already at level {}", hero_id, expect_level);

                let reply = HeroLevelUpReply {
                    hero_id: Some(hero_id),
                    new_level: Some(expect_level),
                };

                let level_push = sonettobuf::HeroLevelUpUpdatePush {
                    hero_id: Some(hero_id),
                    new_level: Some(expect_level),
                    new_rank: Some(hero.record.rank),
                };

                let hero_proto: sonettobuf::HeroInfo = hero.into();
                let hero_push = sonettobuf::HeroUpdatePush {
                    hero_updates: vec![hero_proto],
                };

                drop(conn);

                let mut conn = ctx.lock().await;
                conn.notify(CmdId::HeroLevelUpUpdatePushCmd, level_push)
                    .await?;
                conn.notify(CmdId::HeroHeroUpdatePushCmd, hero_push).await?;
                conn.send_reply(CmdId::HeroLevelUpCmd, reply, 0, req.up_tag)
                    .await?;

                return Ok(());
            } else {
                tracing::warn!(
                    "Invalid level up: expect_level {} < current level {}",
                    expect_level,
                    hero.record.level
                );
                return Err(AppError::InvalidRequest);
            }
        }

        if expect_level > 180 {
            tracing::warn!("Invalid level up: expect_level {} > max 180", expect_level);
            return Err(AppError::InvalidRequest);
        }

        let game_data = data::exceldb::get();

        let character = game_data.character.iter().find(|c| c.id == hero_id);

        let character = match character {
            Some(c) => c,
            None => {
                tracing::error!("Character {} not found in game data", hero_id);
                return Err(AppError::InvalidRequest);
            }
        };

        let hero_rare = character.rare;

        let mut total_costs: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();

        for level in (hero.record.level + 1)..=expect_level {
            let cost_entry = game_data
                .character_cosume
                .iter()
                .find(|c| c.level == level && c.rare == hero_rare);

            let cost_entry = match cost_entry {
                Some(c) => c,
                None => {
                    tracing::error!(
                        "Cost entry not found for level {} rare {} (hero {})",
                        level,
                        hero_rare,
                        hero_id
                    );
                    return Err(AppError::InvalidRequest);
                }
            };

            if cost_entry.cosume.is_empty() {
                continue;
            }

            for cost_part in cost_entry.cosume.split('|') {
                let parts: Vec<&str> = cost_part.split('#').collect();
                if parts.len() >= 3 && parts[0] == "2" {
                    let currency_id: i32 =
                        parts[1].parse().map_err(|_| AppError::InvalidRequest)?;
                    let amount: i32 = parts[2].parse().map_err(|_| AppError::InvalidRequest)?;

                    *total_costs.entry(currency_id).or_insert(0) += amount;
                }
            }
        }

        for (currency_id, amount) in &total_costs {
            let current =
                database::db::game::currencies::get_currency(pool, player_id, *currency_id)
                    .await?
                    .map(|c| c.quantity)
                    .unwrap_or(0);

            if current < *amount {
                tracing::info!(
                    "User {} insufficient currency {} for level up (has {}, needs {})",
                    player_id,
                    currency_id,
                    current,
                    amount
                );

                drop(conn);

                push::send_currency_change_push(ctx.clone(), player_id, vec![(*currency_id, 0)])
                    .await?;

                let data = HeroLevelUpReply {
                    hero_id: Some(hero_id),
                    new_level: Some(hero.record.level),
                };

                let mut conn = ctx.lock().await;
                conn.send_reply(CmdId::HeroLevelUpCmd, data, 0, req.up_tag)
                    .await?;

                return Ok(());
            }
        }

        for (currency_id, amount) in &total_costs {
            database::db::game::currencies::remove_currency(pool, player_id, *currency_id, *amount)
                .await?;
        }

        let level_stats = game_data
            .character_level
            .iter()
            .filter(|l| l.hero_id == hero_id && l.level <= expect_level)
            .max_by_key(|l| l.level);

        let level_stats = match level_stats {
            Some(s) => s,
            None => {
                tracing::error!(
                    "No level stats found for hero {} up to level {}",
                    hero_id,
                    expect_level
                );
                return Err(AppError::InvalidRequest);
            }
        };

        sqlx::query(
            r#"UPDATE heroes
               SET level = ?,
                   base_hp = ?,
                   base_attack = ?,
                   base_defense = ?,
                   base_mdefense = ?,
                   base_technic = ?,
                   ex_cri = ?,
                   ex_recri = ?,
                   ex_cri_dmg = ?,
                   ex_cri_def = ?,
                   ex_add_dmg = ?,
                   ex_drop_dmg = ?
               WHERE uid = ?"#,
        )
        .bind(expect_level)
        .bind(level_stats.hp)
        .bind(level_stats.atk)
        .bind(level_stats.def)
        .bind(level_stats.mdef)
        .bind(level_stats.technic)
        .bind(level_stats.cri)
        .bind(level_stats.recri)
        .bind(level_stats.cri_dmg)
        .bind(level_stats.cri_def)
        .bind(level_stats.add_dmg)
        .bind(level_stats.drop_dmg)
        .bind(hero.record.uid)
        .execute(pool)
        .await?;

        tracing::info!(
            "User {} leveled hero {} from {} to {} using stats from milestone level {} (costs: {:?})",
            player_id,
            hero_id,
            old_level,
            expect_level,
            level_stats.level,
            total_costs
        );

        let consumed: Vec<(i32, i32)> = total_costs.into_iter().collect();
        (player_id, hero.record.rank, consumed)
    };

    if !consumed_currencies.is_empty() {
        push::send_currency_change_push(
            ctx.clone(),
            user_id,
            consumed_currencies.iter().map(|(id, _)| (*id, 0)).collect(),
        )
        .await?;
    }

    let reply = HeroLevelUpReply {
        hero_id: Some(hero_id),
        new_level: Some(expect_level),
    };

    {
        let mut conn = ctx.lock().await;

        let level_push = sonettobuf::HeroLevelUpUpdatePush {
            hero_id: Some(hero_id),
            new_level: Some(expect_level),
            new_rank: Some(new_rank),
        };
        conn.notify(CmdId::HeroLevelUpUpdatePushCmd, level_push)
            .await?;

        let updated_hero = heroes::get_hero_by_hero_id(&conn.state.db, user_id, hero_id).await?;
        let hero_proto: sonettobuf::HeroInfo = updated_hero.into();
        let hero_push = sonettobuf::HeroUpdatePush {
            hero_updates: vec![hero_proto],
        };
        conn.notify(CmdId::HeroHeroUpdatePushCmd, hero_push).await?;

        conn.send_reply(CmdId::HeroLevelUpCmd, reply, 0, req.up_tag)
            .await?;

        tracing::info!(
            "Sent HeroLevelUpUpdatePush and HeroUpdatePush for hero {} to level {}",
            hero_id,
            expect_level
        );
    }
    Ok(())
}
