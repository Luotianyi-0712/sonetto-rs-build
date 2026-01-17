use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::util::inventory::{add_currencies, add_items};
use crate::util::push;
use prost::Message;
use sonettobuf::{CmdId, ReadMailBatchReply, ReadMailBatchRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_read_mail_batch(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = ReadMailBatchRequest::decode(&req.data[..])?;
    let r#type = request.r#type.ok_or(AppError::InvalidRequest)?;

    tracing::info!("Received ReadMailBatchRequest type {}", r#type);

    let (
        user_id,
        incr_ids,
        all_items,
        all_currencies,
        all_equips,
        _, //all heroes
        all_material_changes,
    ) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        let mails: Vec<(i64, String)> = sqlx::query_as(
            "SELECT incr_id, attachment FROM user_mails WHERE user_id = ? AND state = 0 ORDER BY create_time DESC"
        )
        .bind(player_id)
        .fetch_all(pool)
        .await?;

        if mails.is_empty() {
            return Ok(());
        }

        let mail_ids: Vec<i64> = mails.iter().map(|(id, _)| *id).collect();

        let mut total_items = Vec::new();
        let mut total_currencies = Vec::new();
        let mut total_equips = Vec::new();
        let mut total_heroes = Vec::new();
        let mut total_power_items = Vec::new();

        for (_incr_id, attachment) in &mails {
            if !attachment.is_empty() {
                let (items, currencies, equips, heroes, power_items) =
                    crate::state::parse_store_product(attachment);

                total_items.extend(items);
                total_currencies.extend(currencies);
                total_equips.extend(equips);
                total_heroes.extend(heroes);
                total_power_items.extend(power_items);
            }
        }

        let item_ids = if !total_items.is_empty() {
            add_items(pool, player_id, &total_items).await?
        } else {
            vec![]
        };

        let currency_ids = if !total_currencies.is_empty() {
            add_currencies(pool, player_id, &total_currencies).await?
        } else {
            vec![]
        };

        let equip_ids = if !total_equips.is_empty() {
            database::db::game::equipment::add_equipments(
                pool,
                player_id,
                &total_equips
                    .iter()
                    .map(|(id, count)| (*id as i32, *count))
                    .collect::<Vec<_>>(),
            )
            .await?
        } else {
            vec![]
        };

        if !total_power_items.is_empty() {
            database::db::game::items::add_power_items(
                pool,
                player_id,
                &total_power_items
                    .iter()
                    .map(|(id, count)| (*id as i32, *count))
                    .collect::<Vec<_>>(),
            )
            .await?;
        }

        for (hero_id, _count) in &total_heroes {
            let hero_id = *hero_id as i32;

            if database::db::game::heroes::has_hero(pool, player_id, hero_id).await? {
                let duplicate_count =
                    database::db::game::heroes::add_hero_duplicate(pool, player_id, hero_id)
                        .await?;

                tracing::info!(
                    "User {} already has hero {}, granted dupe rewards (duplicate #{})",
                    player_id,
                    hero_id,
                    duplicate_count
                );
            } else {
                database::db::game::heroes::create_hero(pool, player_id, hero_id).await?;
                tracing::info!(
                    "User {} received new hero {} from batch mail",
                    player_id,
                    hero_id
                );
            }
        }

        let now = common::time::ServerTime::now_ms();
        sqlx::query("UPDATE user_mails SET state = 1 WHERE user_id = ? AND state = 0")
            .bind(player_id)
            .execute(pool)
            .await?;

        for mail_id in &mail_ids {
            sqlx::query(
                "INSERT INTO user_mail_history
                 (user_id, mail_incr_id, mail_id, attachment, action, action_time, state_at_action)
                 SELECT user_id, incr_id, mail_id, attachment, 'claimed', ?, 1
                 FROM user_mails
                 WHERE incr_id = ?",
            )
            .bind(now)
            .bind(mail_id)
            .execute(pool)
            .await?;
        }

        tracing::info!(
            "User {} claimed {} mails: {} items, {} currencies, {} equips, {} heroes, {} power items",
            player_id,
            mail_ids.len(),
            total_items.len(),
            total_currencies.len(),
            total_equips.len(),
            total_heroes.len(),
            total_power_items.len()
        );

        let mut material_changes = Vec::new();
        for (item_id, amount) in &total_items {
            material_changes.push((1, *item_id, *amount));
        }
        for (currency_id, amount) in &total_currencies {
            material_changes.push((2, *currency_id as u32, *amount));
        }
        for (equip_id, amount) in &total_equips {
            material_changes.push((9, *equip_id, *amount));
        }
        for (hero_id, amount) in &total_heroes {
            material_changes.push((4, *hero_id, *amount));
        }
        for (power_item_id, amount) in &total_power_items {
            material_changes.push((10, *power_item_id, *amount));
        }

        (
            player_id,
            mail_ids,
            item_ids,
            currency_ids,
            equip_ids,
            total_heroes,
            material_changes,
        )
    };

    let reply = ReadMailBatchReply {
        incr_ids: incr_ids.iter().map(|id| *id as u64).collect(),
    };

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::ReadMailBatchCmd, reply, 0, req.up_tag)
            .await?;
    }

    if !all_items.is_empty() {
        push::send_item_change_push(ctx.clone(), user_id, all_items).await?;
    }

    if !all_currencies.is_empty() {
        push::send_currency_change_push(ctx.clone(), user_id, all_currencies).await?;
    }

    if !all_equips.is_empty() {
        push::send_equip_update_push(ctx.clone(), user_id, all_equips).await?;
    }

    if !all_material_changes.is_empty() {
        push::send_material_change_push(ctx.clone(), all_material_changes, Some(10)).await?;
    }

    Ok(())
}
