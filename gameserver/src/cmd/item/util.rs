use rand::{seq::SliceRandom, thread_rng};
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    state::{get_rewards, parse_item},
};

pub fn process_item_use(
    material_id: u32,
    quantity: i32,
    target_id: Option<u64>,
) -> (Vec<(u32, i32)>, Vec<(i32, i32)>) {
    let is_selector = material_id >= 481000 && material_id <= 481020;

    let is_hero_selector = material_id == 481022;
    if is_hero_selector && target_id.is_some() {
        return (vec![], vec![]);
    }

    if is_selector && target_id.is_some() {
        let (items, currencies) = get_rewards(material_id);
        let target_idx = target_id.unwrap() as usize;

        if let Some(item) = items.get(target_idx) {
            (vec![(item.0, item.1 * quantity)], vec![])
        } else if let Some(currency) = currencies.get(target_idx) {
            (vec![], vec![(currency.0, currency.1 * quantity)])
        } else {
            tracing::warn!(
                "Invalid target_id {} for selector item {}",
                target_idx,
                material_id
            );
            (vec![], vec![])
        }
    } else if target_id.unwrap_or(0) > 0 {
        let target_id_val = target_id.unwrap();
        (vec![(target_id_val as u32, quantity)], vec![])
    } else {
        let game_data = data::exceldb::get();
        let item_cfg = game_data.item.get(material_id as i32);

        if let Some(cfg) = item_cfg {
            if let Some((items, currencies)) = parse_item(&cfg.effect) {
                let final_items: Vec<(u32, i32)> = items
                    .iter()
                    .map(|(id, amt)| (*id, amt * quantity))
                    .collect();
                let final_currencies: Vec<(i32, i32)> = currencies
                    .iter()
                    .map(|(id, amt)| (*id, amt * quantity))
                    .collect();
                (final_items, final_currencies)
            } else {
                let (items, currencies) = get_rewards(material_id);
                let final_items = if items.len() > 1 {
                    let mut rng = thread_rng();
                    let mut selected = Vec::new();
                    for _ in 0..quantity {
                        if let Some(random_item) = items.choose(&mut rng) {
                            selected.push(*random_item);
                        }
                    }
                    selected
                } else {
                    items
                        .iter()
                        .map(|(id, amt)| (*id, amt * quantity))
                        .collect()
                };
                let final_currencies: Vec<(i32, i32)> = currencies
                    .iter()
                    .map(|(id, amt)| (*id, amt * quantity))
                    .collect();
                (final_items, final_currencies)
            }
        } else {
            tracing::warn!("Item {} not found in game data", material_id);
            (vec![], vec![])
        }
    }
}

pub async fn apply_insight_item(
    pool: &SqlitePool,
    player_id: i64,
    uid: i64,
    hero_id: i32,
) -> Result<i32, AppError> {
    let (item_id, quantity): (i32, i32) =
        sqlx::query_as("SELECT item_id, quantity FROM insight_items WHERE uid = ? AND user_id = ?")
            .bind(uid)
            .bind(player_id)
            .fetch_optional(pool)
            .await?
            .ok_or(AppError::InvalidRequest)?;

    if quantity <= 0 {
        tracing::warn!(
            "User {} tried to use insight item with 0 quantity (uid: {})",
            player_id,
            uid
        );
        return Ok(item_id);
    }

    let game_data = data::exceldb::get();
    let insight_data = game_data
        .insight_item
        .iter()
        .find(|i| i.id == item_id)
        .ok_or(AppError::InvalidRequest)?;

    let hero = database::db::game::heroes::get_hero_by_hero_id(pool, player_id, hero_id).await?;

    let target_rank = insight_data.hero_rank + 1;
    let target_level = insight_data
        .effect
        .split('#')
        .nth(1)
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(1);

    sqlx::query("UPDATE heroes SET rank = ?, level = ? WHERE user_id = ? AND hero_id = ?")
        .bind(target_rank)
        .bind(target_level)
        .bind(player_id)
        .bind(hero_id)
        .execute(pool)
        .await?;

    if target_rank >= 3 {
        unlock_insight_skin(pool, player_id, hero_id, hero.record.uid).await?;
    }

    sqlx::query(
        "UPDATE insight_items
         SET quantity = quantity - 1
         WHERE uid = ? AND user_id = ?",
    )
    .bind(uid)
    .bind(player_id)
    .execute(pool)
    .await?;

    Ok(item_id)
}

async fn unlock_insight_skin(
    pool: &SqlitePool,
    player_id: i64,
    hero_id: i32,
    hero_uid: i64,
) -> Result<(), AppError> {
    let game_data = data::exceldb::get();
    let Some(skin) = game_data
        .skin
        .iter()
        .find(|s| s.character_id == hero_id && s.id % 100 == 2 && s.gain_approach == 1)
    else {
        return Ok(());
    };

    let has_skin: Option<i32> =
        sqlx::query_scalar("SELECT 1 FROM hero_all_skins WHERE user_id = ? AND skin_id = ?")
            .bind(player_id)
            .bind(skin.id)
            .fetch_optional(pool)
            .await?;

    if has_skin.is_some() {
        return Ok(());
    }

    sqlx::query("INSERT INTO hero_all_skins (user_id, skin_id) VALUES (?, ?)")
        .bind(player_id)
        .bind(skin.id)
        .execute(pool)
        .await?;

    sqlx::query("INSERT INTO hero_skins (hero_uid, skin, expire_sec) VALUES (?, ?, 0)")
        .bind(hero_uid)
        .bind(skin.id)
        .execute(pool)
        .await?;

    sqlx::query("UPDATE heroes SET skin = ? WHERE uid = ? AND user_id = ?")
        .bind(skin.id)
        .bind(hero_uid)
        .bind(player_id)
        .execute(pool)
        .await?;

    Ok(())
}
