use crate::models::game::{
    block_packages::{BlockInfo, BlockPackage, RoadInfo, SpecialBlock},
    buildings::Building,
};
use anyhow::Result;
use common::time::ServerTime;
use sqlx::SqlitePool;

// Block Packages
pub async fn get_block_packages(pool: &SqlitePool, user_id: i64) -> Result<Vec<BlockPackage>> {
    let packages = sqlx::query_as::<_, BlockPackage>(
        "SELECT user_id, block_package_id, unused_block_ids, used_block_ids
         FROM user_block_packages
         WHERE user_id = ?
         ORDER BY block_package_id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(packages)
}

pub async fn add_block_package(pool: &SqlitePool, user_id: i64, package_id: i32) -> Result<()> {
    sqlx::query(
        "INSERT INTO user_block_packages (user_id, block_package_id, unused_block_ids, used_block_ids)
         VALUES (?, ?, '[]', '[]')
         ON CONFLICT DO NOTHING"
    )
    .bind(user_id)
    .bind(package_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_block_package(
    pool: &SqlitePool,
    user_id: i64,
    package_id: i32,
    unused_block_ids: &[i32],
    used_block_ids: &[i32],
) -> Result<()> {
    let unused_json = serde_json::to_string(unused_block_ids)?;
    let used_json = serde_json::to_string(used_block_ids)?;

    sqlx::query(
        "UPDATE user_block_packages
         SET unused_block_ids = ?, used_block_ids = ?
         WHERE user_id = ? AND block_package_id = ?",
    )
    .bind(unused_json)
    .bind(used_json)
    .bind(user_id)
    .bind(package_id)
    .execute(pool)
    .await?;
    Ok(())
}

// Special Blocks
pub async fn get_special_blocks(pool: &SqlitePool, user_id: i64) -> Result<Vec<SpecialBlock>> {
    let blocks = sqlx::query_as::<_, SpecialBlock>(
        "SELECT user_id, block_id, create_time
         FROM user_special_blocks
         WHERE user_id = ?
         ORDER BY block_id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(blocks)
}

pub async fn add_special_block(pool: &SqlitePool, user_id: i64, block_id: i32) -> Result<()> {
    let create_time = ServerTime::now_ms();
    sqlx::query(
        "INSERT INTO user_special_blocks (user_id, block_id, create_time)
         VALUES (?, ?, ?)
         ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(block_id)
    .bind(create_time)
    .execute(pool)
    .await?;
    Ok(())
}

// Placed Blocks
pub async fn get_blocks(pool: &SqlitePool, user_id: i64) -> Result<Vec<BlockInfo>> {
    let blocks = sqlx::query_as::<_, BlockInfo>(
        "SELECT user_id, block_id, x, y, rotate, water_type, block_color
         FROM user_blocks
         WHERE user_id = ?
         ORDER BY block_id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(blocks)
}

pub async fn save_block(pool: &SqlitePool, block: &BlockInfo) -> Result<()> {
    sqlx::query(
        "INSERT INTO user_blocks (user_id, block_id, x, y, rotate, water_type, block_color)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(user_id, block_id) DO UPDATE SET
             x = excluded.x,
             y = excluded.y,
             rotate = excluded.rotate,
             water_type = excluded.water_type,
             block_color = excluded.block_color",
    )
    .bind(block.user_id)
    .bind(block.block_id)
    .bind(block.x)
    .bind(block.y)
    .bind(block.rotate)
    .bind(block.water_type)
    .bind(block.block_color)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_block(pool: &SqlitePool, user_id: i64, block_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM user_blocks WHERE user_id = ? AND block_id = ?")
        .bind(user_id)
        .bind(block_id)
        .execute(pool)
        .await?;
    Ok(())
}

// Buildings
pub async fn get_buildings(pool: &SqlitePool, user_id: i64) -> Result<Vec<Building>> {
    let buildings = sqlx::query_as::<_, Building>(
        "SELECT uid, user_id, define_id, in_use, x, y, rotate, level, created_at, updated_at
         FROM user_buildings
         WHERE user_id = ?
         ORDER BY uid",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(buildings)
}

pub async fn save_building(pool: &SqlitePool, building: &Building) -> Result<i64> {
    let result = sqlx::query(
        "INSERT INTO user_buildings (user_id, define_id, in_use, x, y, rotate, level, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(uid) DO UPDATE SET
             define_id = excluded.define_id,
             in_use = excluded.in_use,
             x = excluded.x,
             y = excluded.y,
             rotate = excluded.rotate,
             level = excluded.level",
    )
    .bind(building.user_id)
    .bind(building.define_id)
    .bind(building.in_use)
    .bind(building.x)
    .bind(building.y)
    .bind(building.rotate)
    .bind(building.level)
    .bind(common::time::ServerTime::now_ms())
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn delete_building(pool: &SqlitePool, user_id: i64, uid: i64) -> Result<()> {
    sqlx::query("DELETE FROM user_buildings WHERE user_id = ? AND uid = ?")
        .bind(user_id)
        .bind(uid)
        .execute(pool)
        .await?;
    Ok(())
}

// Roads
pub async fn get_roads(pool: &SqlitePool, user_id: i64) -> Result<Vec<RoadInfo>> {
    let roads = sqlx::query_as::<_, RoadInfo>(
        "SELECT user_id, id, from_type, to_type, road_points, critter_uid,
                building_uid, building_define_id, skin_id, block_clean_type
         FROM user_roads
         WHERE user_id = ?
         ORDER BY id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(roads)
}

pub async fn save_road(pool: &SqlitePool, road: &RoadInfo) -> Result<()> {
    sqlx::query(
        "INSERT INTO user_roads (user_id, id, from_type, to_type, road_points,
                                  critter_uid, building_uid, building_define_id,
                                  skin_id, block_clean_type)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(user_id, id) DO UPDATE SET
             from_type = excluded.from_type,
             to_type = excluded.to_type,
             road_points = excluded.road_points,
             critter_uid = excluded.critter_uid,
             building_uid = excluded.building_uid,
             building_define_id = excluded.building_define_id,
             skin_id = excluded.skin_id,
             block_clean_type = excluded.block_clean_type",
    )
    .bind(road.user_id)
    .bind(road.id)
    .bind(road.from_type)
    .bind(road.to_type)
    .bind(&road.road_points)
    .bind(road.critter_uid)
    .bind(road.building_uid)
    .bind(road.building_define_id)
    .bind(road.skin_id)
    .bind(road.block_clean_type)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_road(pool: &SqlitePool, user_id: i64, id: i32) -> Result<()> {
    sqlx::query("DELETE FROM user_roads WHERE user_id = ? AND id = ?")
        .bind(user_id)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// Room State
pub async fn get_room_reset_state(pool: &SqlitePool, user_id: i64) -> Result<bool> {
    let result: Option<(bool,)> =
        sqlx::query_as("SELECT is_reset FROM user_room_state WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    Ok(result.map(|(is_reset,)| is_reset).unwrap_or(false))
}

pub async fn set_room_reset_state(pool: &SqlitePool, user_id: i64, is_reset: bool) -> Result<()> {
    let now = ServerTime::now_ms();
    sqlx::query(
        "INSERT INTO user_room_state (user_id, is_reset, last_reset_time)
         VALUES (?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET
             is_reset = excluded.is_reset,
             last_reset_time = excluded.last_reset_time",
    )
    .bind(user_id)
    .bind(is_reset)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}
