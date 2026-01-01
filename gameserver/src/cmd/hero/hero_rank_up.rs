use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::utils::push;
use database::db::game::heroes;
use prost::Message;
use sonettobuf::{CmdId, HeroRankUpReply, HeroRankUpRequest, HeroUpdatePush};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_rank_up(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = HeroRankUpRequest::decode(&req.data[..])?;
    tracing::info!("Received HeroRankUpRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let (user_id, new_rank, consumed_items, consumed_currencies) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &ctx_guard.state.db;

        let hero = heroes::get_hero_by_hero_id(pool, player_id, hero_id).await?;
        let current_rank = hero.record.rank;
        let target_rank = current_rank + 1;

        let game_data = data::exceldb::get();

        let rank_data = game_data
            .character_rank
            .iter()
            .find(|r| r.hero_id == hero_id && r.rank == target_rank);

        let rank_data = match rank_data {
            Some(r) => r,
            None => {
                tracing::info!(
                    "User {} hero {} already at max rank {}",
                    player_id,
                    hero_id,
                    current_rank
                );

                let reply = HeroRankUpReply {
                    hero_id: Some(hero_id),
                    new_rank: Some(current_rank),
                };

                drop(ctx_guard);

                let mut ctx_guard = ctx.lock().await;
                let hero_proto: sonettobuf::HeroInfo = hero.into();
                ctx_guard
                    .send_push(
                        CmdId::HeroHeroUpdatePushCmd,
                        HeroUpdatePush {
                            hero_updates: vec![hero_proto],
                        },
                    )
                    .await?;
                ctx_guard
                    .send_reply(CmdId::HeroRankUpCmd, reply, 0, req.up_tag)
                    .await?;

                return Ok(());
            }
        };

        if !rank_data.requirement.is_empty() {
            let req_parts: Vec<&str> = rank_data.requirement.split('#').collect();
            if req_parts.len() >= 2 && req_parts[0] == "1" {
                let required_level: i32 =
                    req_parts[1].parse().map_err(|_| AppError::InvalidRequest)?;

                if hero.record.level != required_level {
                    tracing::info!(
                        "Hero {} level {} does not match requirement {} for rank {} (retry)",
                        hero_id,
                        hero.record.level,
                        required_level,
                        target_rank
                    );

                    let reply = HeroRankUpReply {
                        hero_id: Some(hero_id),
                        new_rank: Some(current_rank),
                    };

                    drop(ctx_guard);

                    let mut ctx_guard = ctx.lock().await;
                    let hero_proto: sonettobuf::HeroInfo = hero.into();
                    ctx_guard
                        .send_push(
                            CmdId::HeroHeroUpdatePushCmd,
                            HeroUpdatePush {
                                hero_updates: vec![hero_proto],
                            },
                        )
                        .await?;
                    ctx_guard
                        .send_reply(CmdId::HeroRankUpCmd, reply, 0, req.up_tag)
                        .await?;

                    return Ok(());
                }
            }
        }

        let mut cost_items = Vec::new();
        let mut cost_currencies = Vec::new();

        if !rank_data.consume.is_empty() {
            for cost_part in rank_data.consume.split('|') {
                let parts: Vec<&str> = cost_part.split('#').collect();
                if parts.len() >= 3 {
                    match parts[0] {
                        "1" => {
                            let item_id: u32 =
                                parts[1].parse().map_err(|_| AppError::InvalidRequest)?;
                            let amount: i32 =
                                parts[2].parse().map_err(|_| AppError::InvalidRequest)?;
                            cost_items.push((item_id, amount));
                        }
                        "2" => {
                            let currency_id: i32 =
                                parts[1].parse().map_err(|_| AppError::InvalidRequest)?;
                            let amount: i32 =
                                parts[2].parse().map_err(|_| AppError::InvalidRequest)?;
                            cost_currencies.push((currency_id, amount));
                        }
                        _ => {}
                    }
                }
            }
        }

        for (item_id, amount) in &cost_items {
            let current = database::db::game::items::get_item(pool, player_id, *item_id)
                .await?
                .map(|i| i.quantity)
                .unwrap_or(0);

            if current < *amount {
                tracing::info!(
                    "User {} insufficient item {} (has {}, needs {})",
                    player_id,
                    item_id,
                    current,
                    amount
                );

                drop(ctx_guard);
                push::send_item_change_push(ctx.clone(), player_id, vec![*item_id], vec![], vec![])
                    .await?;

                let mut ctx_guard = ctx.lock().await;
                ctx_guard
                    .send_reply(
                        CmdId::HeroRankUpCmd,
                        HeroRankUpReply {
                            hero_id: Some(hero_id),
                            new_rank: Some(current_rank),
                        },
                        0,
                        req.up_tag,
                    )
                    .await?;

                return Ok(());
            }
        }

        for (currency_id, amount) in &cost_currencies {
            let current =
                database::db::game::currencies::get_currency(pool, player_id, *currency_id)
                    .await?
                    .map(|c| c.quantity)
                    .unwrap_or(0);

            if current < *amount {
                tracing::info!(
                    "User {} insufficient currency {} (has {}, needs {})",
                    player_id,
                    currency_id,
                    current,
                    amount
                );

                drop(ctx_guard);
                push::send_currency_change_push(ctx.clone(), player_id, vec![(*currency_id, 0)])
                    .await?;

                let mut ctx_guard = ctx.lock().await;
                ctx_guard
                    .send_reply(
                        CmdId::HeroRankUpCmd,
                        HeroRankUpReply {
                            hero_id: Some(hero_id),
                            new_rank: Some(current_rank),
                        },
                        0,
                        req.up_tag,
                    )
                    .await?;

                return Ok(());
            }
        }

        for (item_id, amount) in &cost_items {
            database::db::game::items::remove_item_quantity(pool, player_id, *item_id, *amount)
                .await?;
        }

        for (currency_id, amount) in &cost_currencies {
            database::db::game::currencies::remove_currency(pool, player_id, *currency_id, *amount)
                .await?;
        }

        sqlx::query("UPDATE heroes SET rank = ?, level = 1 WHERE uid = ? AND user_id = ?")
            .bind(target_rank)
            .bind(hero.record.uid)
            .bind(player_id)
            .execute(pool)
            .await?;

        tracing::info!(
            "User {} ranked up hero {} from rank {} to {} (level reset to 1)",
            player_id,
            hero_id,
            current_rank,
            target_rank
        );

        if target_rank == 3 {
            let insight_skin = game_data
                .skin
                .iter()
                .find(|s| s.character_id == hero_id && s.id % 100 == 2 && s.gain_approach == 1);

            if let Some(skin) = insight_skin {
                let has_skin: Option<i32> = sqlx::query_scalar(
                    "SELECT 1 FROM hero_all_skins WHERE user_id = ? AND skin_id = ?",
                )
                .bind(player_id)
                .bind(skin.id)
                .fetch_optional(pool)
                .await?;

                if has_skin.is_none() {
                    sqlx::query("INSERT INTO hero_all_skins (user_id, skin_id) VALUES (?, ?)")
                        .bind(player_id)
                        .bind(skin.id)
                        .execute(pool)
                        .await?;

                    sqlx::query(
                        "INSERT INTO hero_skins (hero_uid, skin, expire_sec) VALUES (?, ?, ?)",
                    )
                    .bind(hero.record.uid)
                    .bind(skin.id)
                    .bind(0)
                    .execute(pool)
                    .await?;

                    sqlx::query("UPDATE heroes SET skin = ? WHERE uid = ? AND user_id = ?")
                        .bind(skin.id)
                        .bind(hero.record.uid)
                        .bind(player_id)
                        .execute(pool)
                        .await?;

                    tracing::info!(
                        "User {} unlocked and equipped Insight II skin {} for hero {}",
                        player_id,
                        skin.id,
                        hero_id
                    );
                }
            }
        }

        (player_id, target_rank, cost_items, cost_currencies)
    };

    if !consumed_items.is_empty() {
        push::send_item_change_push(
            ctx.clone(),
            user_id,
            consumed_items.iter().map(|(id, _)| *id).collect(),
            vec![],
            vec![],
        )
        .await?;
    }

    if !consumed_currencies.is_empty() {
        push::send_currency_change_push(
            ctx.clone(),
            user_id,
            consumed_currencies.iter().map(|(id, _)| (*id, 0)).collect(),
        )
        .await?;
    }

    let mut ctx_guard = ctx.lock().await;
    let updated_hero = heroes::get_hero_by_hero_id(&ctx_guard.state.db, user_id, hero_id).await?;

    ctx_guard
        .send_push(
            CmdId::HeroHeroUpdatePushCmd,
            HeroUpdatePush {
                hero_updates: vec![updated_hero.into()],
            },
        )
        .await?;

    ctx_guard
        .send_reply(
            CmdId::HeroRankUpCmd,
            HeroRankUpReply {
                hero_id: Some(hero_id),
                new_rank: Some(new_rank),
            },
            0,
            req.up_tag,
        )
        .await?;

    tracing::info!("Hero {} ranked up to {}", hero_id, new_rank);

    Ok(())
}
