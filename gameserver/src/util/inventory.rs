use crate::error::AppError;
use sqlx::SqlitePool;

pub async fn add_items(
    pool: &SqlitePool,
    player_id: i64,
    items: &[(u32, i32)],
) -> Result<Vec<u32>, AppError> {
    let mut changed_ids = Vec::new();
    for (item_id, quantity) in items {
        database::db::game::items::add_item_quantity(pool, player_id, *item_id, *quantity).await?;
        changed_ids.push(*item_id);
    }
    Ok(changed_ids)
}

pub async fn add_currencies(
    pool: &SqlitePool,
    player_id: i64,
    currencies: &[(i32, i32)],
) -> Result<Vec<(i32, i32)>, AppError> {
    let mut changes = Vec::new();
    for (currency_id, amount) in currencies {
        database::db::game::currencies::add_currency(pool, player_id, *currency_id, *amount)
            .await?;
        changes.push((*currency_id, *amount));
    }
    Ok(changes)
}
