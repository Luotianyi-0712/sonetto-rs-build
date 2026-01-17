use serde::{Deserialize, Serialize};
use sonettobuf;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct SpecialBlock {
    pub user_id: i64,
    pub block_id: i32,
    pub create_time: i32,
}

impl From<SpecialBlock> for sonettobuf::SpecialBlockInfo {
    fn from(b: SpecialBlock) -> Self {
        sonettobuf::SpecialBlockInfo {
            block_id: Some(b.block_id),
            create_time: Some(b.create_time),
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BlockInfo {
    pub user_id: i64,
    pub block_id: i32,
    pub x: i32,
    pub y: i32,
    pub rotate: i32,
    pub water_type: i32,
    pub block_color: i32,
}

impl From<BlockInfo> for sonettobuf::BlockInfo {
    fn from(b: BlockInfo) -> Self {
        sonettobuf::BlockInfo {
            block_id: Some(b.block_id),
            x: Some(b.x),
            y: Some(b.y),
            rotate: Some(b.rotate),
            water_type: Some(b.water_type),
            block_color: Some(b.block_color),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct BlockPackage {
    pub user_id: i64,
    pub block_package_id: i32,
    pub unused_block_ids: String,
    pub used_block_ids: String,
}

impl From<BlockPackage> for sonettobuf::BlockPackageInfo {
    fn from(b: BlockPackage) -> Self {
        let un_use_block_ids: Vec<i32> =
            serde_json::from_str(&b.unused_block_ids).unwrap_or_default();

        let use_block_ids: Vec<i32> = serde_json::from_str(&b.used_block_ids).unwrap_or_default();

        sonettobuf::BlockPackageInfo {
            block_package_id: Some(b.block_package_id),
            un_use_block_ids,
            use_block_ids,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadPoint {
    pub x: i32,
    pub y: i32,
}

impl From<RoadPoint> for sonettobuf::RoadPoint {
    fn from(p: RoadPoint) -> Self {
        sonettobuf::RoadPoint {
            x: Some(p.x),
            y: Some(p.y),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct RoadInfo {
    pub user_id: i64,
    pub id: i32,
    pub from_type: i32,
    pub to_type: i32,
    pub road_points: String,
    pub critter_uid: i64,
    pub building_uid: i64,
    pub building_define_id: i32,
    pub skin_id: i32,
    pub block_clean_type: i32,
}

impl From<RoadInfo> for sonettobuf::RoadInfo {
    fn from(r: RoadInfo) -> Self {
        let road_points: Vec<RoadPoint> = serde_json::from_str(&r.road_points).unwrap_or_default();

        sonettobuf::RoadInfo {
            id: Some(r.id),
            from_type: Some(r.from_type),
            to_type: Some(r.to_type),
            road_points: road_points.into_iter().map(Into::into).collect(),
            critter_uid: Some(r.critter_uid),
            building_uid: Some(r.building_uid),
            building_define_id: Some(r.building_define_id),
            skin_id: Some(r.skin_id),
            block_clean_type: Some(r.block_clean_type),
        }
    }
}
