use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::{
    db::game::hero_group_snapshots::{self, sync_snapshot_to_common},
    models::game::hero_groups,
};
use prost::Message;
use sonettobuf::{CmdId, SetHeroGroupSnapshotReply, SetHeroGroupSnapshotRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_set_hero_group_snapshot(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = SetHeroGroupSnapshotRequest::decode(&req.data[..])?;
    tracing::info!("Received SetHeroGroupSnapshotRequest: {:?}", request);

    let snapshot_id = request.snapshot_id.ok_or(AppError::InvalidRequest)?;
    let snapshot_sub_id = request.snapshot_sub_id.unwrap_or(0);
    let fight_group = request.fight_group.ok_or(AppError::InvalidRequest)?;

    let (player_id, pool) = {
        let conn = ctx.lock().await;
        (
            conn.player_id.ok_or(AppError::NotLoggedIn)?,
            conn.state.db.clone(),
        )
    };

    let hero_group = hero_groups::HeroGroupInfo {
        group_id: snapshot_sub_id,
        hero_list: {
            let mut heroes: Vec<i64> = fight_group
                .hero_list
                .into_iter()
                .filter(|&uid| uid != 0)
                .collect();
            heroes.extend(
                fight_group
                    .sub_hero_list
                    .into_iter()
                    .filter(|&uid| uid != 0),
            );
            heroes
        },
        name: String::new(),
        cloth_id: fight_group.cloth_id.unwrap_or(1),
        equips: fight_group
            .equips
            .into_iter()
            .enumerate()
            .filter(|(_, e)| e.hero_uid.unwrap_or(0) != 0)
            .map(|(index, e)| hero_groups::HeroGroupEquip {
                index: index as i32,
                equip_uids: e.equip_uid.into_iter().filter(|&uid| uid != 0).collect(),
            })
            .collect(),
        activity104_equips: fight_group
            .activity104_equips
            .into_iter()
            .enumerate()
            .filter(|(_, e)| {
                let uid = e.hero_uid.unwrap_or(0);
                uid != 0
            })
            .map(|(index, e)| hero_groups::HeroGroupEquip {
                index: index as i32,
                equip_uids: e.equip_uid.into_iter().filter(|&uid| uid != 0).collect(),
            })
            .collect(),
        assist_boss_id: fight_group.assist_boss_id.unwrap_or(0),
    };

    hero_group_snapshots::save_hero_group_snapshot(
        &pool,
        player_id,
        snapshot_id,
        vec![hero_group.clone()],
        vec![snapshot_sub_id],
    )
    .await?;

    tracing::info!(
        "Saved hero group snapshot {} (sub {}) for user {}",
        snapshot_id,
        snapshot_sub_id,
        player_id
    );

    sync_snapshot_to_common(&pool, player_id, &hero_group).await?;

    let data = SetHeroGroupSnapshotReply {
        snapshot_id: Some(snapshot_id),
        snapshot_sub_id: Some(snapshot_sub_id),
        group_info: Some(hero_group.into()),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::SetHeroGroupSnapshotCmd, data, 0, req.up_tag)
        .await?;

    Ok(())
}
