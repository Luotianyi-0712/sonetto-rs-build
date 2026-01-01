use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::utils::inventory::{add_currencies, add_items};
use crate::utils::push::{self, send_red_dot_push};
#[allow(unused_imports)]
use sonettobuf::{CmdId, GainSpecialBlockPush, GetMonthCardInfoReply, MonthCardInfo};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_month_card_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (can_claim, current_time, card_infos) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &ctx_guard.state.db;
        let current_time = common::time::ServerTime::now_ms();
        let current_server_day = common::time::ServerTime::server_day(current_time);

        let active_cards: Vec<(i32, i64)> = sqlx::query_as(
            "SELECT card_id, end_time FROM user_month_card_history
             WHERE user_id = ? AND end_time > ?
             ORDER BY card_id",
        )
        .bind(player_id)
        .bind(current_time / 1000)
        .fetch_all(pool)
        .await?;

        let mut card_infos = Vec::new();
        let can_claim;

        if active_cards.is_empty() {
            can_claim = false;
        } else {
            let claimed_today: Option<i32> = sqlx::query_scalar(
                "SELECT 1 FROM user_month_card_days
                 WHERE user_id = ? AND server_day = ?",
            )
            .bind(player_id)
            .bind(current_server_day)
            .fetch_optional(pool)
            .await?;

            can_claim = claimed_today.is_none();

            for (card_id, end_time) in active_cards {
                card_infos.push(MonthCardInfo {
                    id: Some(card_id),
                    expire_time: Some(end_time as i32),
                    has_get_bonus: Some(!can_claim),
                });
            }
        }

        (can_claim, current_time, card_infos)
    };

    if can_claim && !card_infos.is_empty() {
        tracing::info!("Claiming month card daily bonus");
        let (player_id, all_rewards, changed_items, changed_currencies) = {
            let ctx_guard = ctx.lock().await;
            let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
            let pool = &ctx_guard.state.db;
            let current_server_day = common::time::ServerTime::server_day(current_time);
            let game_data = data::exceldb::get();
            let mut all_rewards = String::new();

            for card_info in &card_infos {
                let card_id = card_info.id.unwrap() as i32;
                if let Some(month_card) = game_data.month_card.iter().find(|m| m.id == card_id) {
                    if !all_rewards.is_empty() {
                        all_rewards.push('|');
                    }
                    all_rewards.push_str(&month_card.daily_bonus);
                }
            }

            let (items, currencies, _, _, power_items, _) =
                crate::state::parse_store_product(&all_rewards);

            let item_ids = if !items.is_empty() {
                add_items(pool, player_id, &items).await?
            } else {
                vec![]
            };

            let currency_ids = if !currencies.is_empty() {
                add_currencies(pool, player_id, &currencies).await?
            } else {
                vec![]
            };

            if !power_items.is_empty() {
                database::db::game::items::add_power_items(
                    pool,
                    player_id,
                    &power_items
                        .iter()
                        .map(|(id, count)| (*id as i32, *count))
                        .collect::<Vec<_>>(),
                )
                .await?;
            }

            sqlx::query(
                "INSERT OR IGNORE INTO user_month_card_days (user_id, server_day, day_of_month)
                 VALUES (?, ?, 1)",
            )
            .bind(player_id)
            .bind(current_server_day)
            .execute(pool)
            .await?;

            (player_id, all_rewards, item_ids, currency_ids)
        };

        {
            let mut ctx_guard = ctx.lock().await;
            ctx_guard
                .update_and_save_player_state(|state| {
                    state.claim_month_card(current_time);
                    state.mark_activity_pushes_sent(current_time);
                })
                .await?;
        }

        if !changed_items.is_empty() {
            push::send_item_change_push(ctx.clone(), player_id, changed_items, vec![], vec![])
                .await?;
        }

        if !changed_currencies.is_empty() {
            push::send_currency_change_push(ctx.clone(), player_id, changed_currencies).await?;
        }

        let mut material_changes = Vec::new();
        let (items, currencies, equips, heroes, power_items, _) =
            crate::state::parse_store_product(&all_rewards);

        for (item_id, amount) in items {
            material_changes.push((1, item_id, amount));
        }
        for (currency_id, amount) in currencies {
            material_changes.push((2, currency_id as u32, amount));
        }
        for (equip_id, amount) in equips {
            material_changes.push((9, equip_id, amount));
        }
        for (hero_id, amount) in heroes {
            material_changes.push((4, hero_id, amount));
        }
        for (power_item_id, amount) in power_items {
            material_changes.push((10, power_item_id, amount));
        }

        if !material_changes.is_empty() {
            push::send_material_change_push(ctx.clone(), material_changes, Some(10)).await?;
        }

        send_red_dot_push(Arc::clone(&ctx), player_id, Some(vec![1040])).await?;
    } else if !card_infos.is_empty() {
        tracing::info!("Month card already claimed today");
    }

    let reply = GetMonthCardInfoReply { infos: card_infos };
    {
        let mut ctx_guard = ctx.lock().await;
        ctx_guard
            .send_reply(CmdId::GetMonthCardInfoCmd, reply, 0, req.up_tag)
            .await?;
    }
    Ok(())
}
