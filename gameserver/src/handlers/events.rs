use crate::network::packet::ClientPacket;
use crate::send_reply;
use crate::state::ConnectionContext;
use crate::{
    error::AppError,
    util::{data_loader::GameDataLoader, push},
};
use database::db::game::activity101;
use database::models::game::{
    currencies::UserCurrencyModel, heros::UserHeroModel, items::UserItemModel,
};

use prost::Message;
use sonettobuf::{
    Act101Info, Act160GetInfoReply, Act165GetInfoReply, Act212BonusNo, Act212InfoNo, CmdId,
    Get101BonusReply, Get101BonusRequest, Get101InfosReply, Get101InfosRequest,
    GetAct125InfosReply, GetAct125InfosRequest, GetAct208InfoReply, GetAct209InfoReply,
    GetAct212InfoReply, GetActivityInfosReply,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_activity_infos(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetActivityInfosCmd,
        GetActivityInfosReply,
        "activity/activity_infos.json"
    );
    Ok(())
}

pub async fn on_get101_bonus(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = Get101BonusRequest::decode(&req.data[..])?;
    let activity_id = request.activity_id.ok_or(AppError::InvalidRequest)?;
    let day_id = request.id.ok_or(AppError::InvalidRequest)?;
    let (player_id, pool) = {
        let conn = ctx.lock().await;
        (
            conn.player_id.ok_or(AppError::NotLoggedIn)?,
            conn.state.db.clone(),
        )
    };
    let now = common::time::ServerTime::now_ms();
    let game_data = config::configs::get();

    let activity_config = game_data
        .activity101
        .iter()
        .find(|a| a.activity_id == activity_id && a.id == day_id as i32)
        .ok_or(AppError::InvalidRequest)?;

    {
        let conn = ctx.lock().await;
        if let Some(state) = &conn.player_state {
            tracing::debug!("Get101Bonus");
            tracing::debug!("last_daily_reward_time: {:?}", state.last_daily_reward_time);
            tracing::debug!("ServerTime: {:?}", now);
            tracing::debug!(
                "CurrServerDay: {:?}",
                common::time::ServerTime::server_day(now)
            );
            tracing::debug!(
                "LastServerDay: {:?}",
                state
                    .last_daily_reward_time
                    .map(common::time::ServerTime::server_day)
            );
            tracing::debug!("IsNewRewardDay: {:?}", state.is_new_reward_day(now));
        }
    }

    let claimed_at: Option<i64> = sqlx::query_scalar(
        "SELECT claimed_at
         FROM user_activity101_claims
         WHERE user_id = ? AND activity_id = ? AND day_id = ?",
    )
    .bind(player_id)
    .bind(activity_id)
    .bind(day_id)
    .fetch_optional(&pool)
    .await?
    .flatten();

    if claimed_at.is_some() {
        tracing::warn!(
            "User {} already claimed day {} for activity {}",
            player_id,
            day_id,
            activity_id
        );
        let reply = Get101BonusReply {
            activity_id: Some(activity_id),
            id: Some(day_id),
        };
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::Get101BonusCmd, reply, 0, req.up_tag)
            .await?;
        return Ok(());
    }

    activity101::claim_activity101_day(&pool, player_id, activity_id, day_id as i32).await?;

    {
        let mut conn = ctx.lock().await;
        conn.update_and_save_player_state(|state| {
            state.mark_daily_reward_claimed(now);
        })
        .await?;
    }

    // Parse rewards from activity config
    // Format: "type#item_id#quantity"
    let bonus_parts: Vec<&str> = activity_config.bonus.split('#').collect();
    if bonus_parts.len() != 3 {
        return Err(AppError::InvalidRequest);
    }

    let reward_type = bonus_parts[0]
        .parse::<u32>()
        .map_err(|_| AppError::InvalidRequest)?;
    let item_id = bonus_parts[1]
        .parse::<u32>()
        .map_err(|_| AppError::InvalidRequest)?;
    let quantity = bonus_parts[2]
        .parse::<i32>()
        .map_err(|_| AppError::InvalidRequest)?;

    // Separate reward containers based on type
    let mut item_rewards = vec![];
    let mut currency_rewards = vec![];
    let mut equip_rewards = vec![];
    let mut hero_rewards = vec![];
    let mut power_item_rewards = vec![];
    let mut insight_item_rewards = vec![];

    match reward_type {
        1 => item_rewards.push((item_id, quantity)), // Items
        2 | 13 => currency_rewards.push((item_id as i32, quantity)), // Currency / Shop currency
        4 => hero_rewards.push((item_id, quantity)), // Heroes
        5 => {
            // Skins
            tracing::warn!("Skin rewards not yet implemented for activity101");
        }
        9 => equip_rewards.push((item_id, quantity)), // Psychubes/Equipment
        10 => power_item_rewards.push((item_id, quantity)), // Power items/energy
        11 => item_rewards.push((item_id, quantity)), // Room Buildings
        14 => item_rewards.push((item_id, quantity)), // Special blocks
        24 => insight_item_rewards.push((item_id, quantity)), // Insight items
        _ => {
            tracing::warn!("Unknown reward type: {}", reward_type);
            return Err(AppError::InvalidRequest);
        }
    }

    let (
        changed_item_ids,
        changed_currency_ids,
        changed_equip_ids,
        hero_dupe_items,
        hero_dupe_currencies,
    ) = {
        let hero = UserHeroModel::new(player_id, pool.clone());
        let item = UserItemModel::new(player_id, pool.clone());
        let currency = UserCurrencyModel::new(player_id, pool.clone());

        let mut item_ids = if !item_rewards.is_empty() {
            item.create_items(&item_rewards).await?
        } else {
            vec![]
        };

        let mut currency_ids = if !currency_rewards.is_empty() {
            currency.create_currencies(&currency_rewards).await?
        } else {
            vec![]
        };

        let equip_ids = if !equip_rewards.is_empty() {
            database::db::game::equipment::add_equipments(
                &pool,
                player_id,
                &equip_rewards
                    .iter()
                    .map(|(id, count)| (*id as i32, *count))
                    .collect::<Vec<_>>(),
            )
            .await?
        } else {
            Vec::new()
        };

        if !power_item_rewards.is_empty() {
            item.create_power_items(
                &power_item_rewards
                    .iter()
                    .map(|(id, count)| (*id as i32, *count))
                    .collect::<Vec<_>>(),
            )
            .await?;
        }

        if !insight_item_rewards.is_empty() {
            let insight_item_ids = item
                .create_insight_items(
                    &insight_item_rewards
                        .iter()
                        .map(|(id, count)| (*id as i32, *count))
                        .collect::<Vec<_>>(),
                )
                .await?;
            item_ids.extend(insight_item_ids);
        }

        let mut hero_dupe_items = Vec::new();
        let mut hero_dupe_currencies = Vec::new();

        for (hero_id, _count) in &hero_rewards {
            let hero_id = *hero_id as i32;

            if hero.has_hero(hero_id).await? {
                let duplicate_count = hero.add_hero_duplicate(hero_id).await?;

                let (dupe_items, dupe_currencies) =
                    crate::state::grant_dupe_rewards(hero_id, duplicate_count).await?;

                hero_dupe_items.extend(dupe_items);
                hero_dupe_currencies.extend(dupe_currencies);

                tracing::info!(
                    "User {} already has hero {}, granted dupe rewards (duplicate #{})",
                    player_id,
                    hero_id,
                    duplicate_count
                );
            } else {
                hero.create_hero(hero_id).await?;
                tracing::info!("User {} received new hero {}", player_id, hero_id);
            }
        }

        if !hero_dupe_items.is_empty() {
            let dupe_item_ids = item.create_items(&hero_dupe_items).await?;
            item_ids.extend(dupe_item_ids);
        }
        if !hero_dupe_currencies.is_empty() {
            let dupe_currency_ids = currency.create_currencies(&hero_dupe_currencies).await?;
            currency_ids.extend(dupe_currency_ids);
        }

        (
            item_ids,
            currency_ids,
            equip_ids,
            hero_dupe_items,
            hero_dupe_currencies,
        )
    };

    tracing::info!(
        "User {} claimed day {} for activity {}: reward type {}#{}#{}",
        player_id,
        day_id,
        activity_id,
        reward_type,
        item_id,
        quantity
    );

    // Build material rewards for popup notification
    let mut material_rewards = Vec::new();

    for (item_id, amount) in &item_rewards {
        material_rewards.push((1, *item_id, *amount));
    }
    for (item_id, amount) in &hero_dupe_items {
        material_rewards.push((1, *item_id, *amount));
    }
    for (currency_id, amount) in &currency_rewards {
        material_rewards.push((2, *currency_id as u32, *amount));
    }
    for (currency_id, amount) in &hero_dupe_currencies {
        material_rewards.push((2, *currency_id as u32, *amount));
    }
    for (equip_id, amount) in &equip_rewards {
        material_rewards.push((9, *equip_id, *amount));
    }
    for (power_item_id, amount) in &power_item_rewards {
        material_rewards.push((10, *power_item_id, *amount));
    }
    for (insight_item_id, amount) in &insight_item_rewards {
        material_rewards.push((24, *insight_item_id, *amount));
    }

    if hero_dupe_items.is_empty() && hero_dupe_currencies.is_empty() {
        for (hero_id, amount) in &hero_rewards {
            material_rewards.push((4, *hero_id, *amount));
        }
    }

    if !changed_item_ids.is_empty() {
        push::send_item_change_push(
            ctx.clone(),
            player_id,
            changed_item_ids.into_iter().map(|id| id as u32).collect(),
            vec![],
            vec![],
        )
        .await?;
    }

    if !changed_currency_ids.is_empty() {
        push::send_currency_change_push(ctx.clone(), player_id, changed_currency_ids).await?;
    }

    if !changed_equip_ids.is_empty() {
        push::send_equip_update_push_by_uid(ctx.clone(), player_id, &changed_equip_ids).await?;
    }

    push::send_red_dot_push(ctx.clone(), player_id, Some(vec![2240])).await?;

    if !material_rewards.is_empty() {
        push::send_material_change_push(ctx.clone(), material_rewards, Some(25)).await?; // 25 = activity source
    }

    push::send_red_dot_push(ctx.clone(), player_id, Some(vec![1010])).await?;
    push::send_red_dot_push(ctx.clone(), player_id, Some(vec![30558, 30557])).await?;

    let reply = Get101BonusReply {
        activity_id: Some(activity_id),
        id: Some(day_id),
    };
    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::Get101BonusCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_get101_infos(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = Get101InfosRequest::decode(&req.data[..])?;
    let activity_id = request.activity_id.unwrap_or(13108);

    tracing::info!("Requested activity_id: {}", activity_id);

    let (infos, login_count, got_once_bonus) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        activity101::get_activity101_info(&conn.state.db, player_id, activity_id).await?
    };

    let reply = Get101InfosReply {
        infos: infos
            .into_iter()
            .map(|(id, state)| Act101Info {
                id: Some(id as u32),
                state: Some(state as u32),
            })
            .collect(),
        sp_infos: vec![],
        login_count: Some(login_count as u32),
        activity_id: Some(activity_id),
        got_once_bonus: Some(got_once_bonus),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::Get101InfosCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_get_act125_infos(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = GetAct125InfosRequest::decode(&req.data[..])?;
    let activity_id = request.activity_id.unwrap_or(0);

    tracing::info!("Requested activity_id: {}", activity_id);

    let path = match activity_id {
        13116 => "activity125/activity125_infos_13116.json",
        13005 => "activity125/activity125_infos_13005.json",
        _ => {
            tracing::warn!("Unknown activity_id: {}, using default", activity_id);
            "activity125/activity125_infos_13116.json"
        }
    };

    let reply: GetAct125InfosReply = GameDataLoader::load_struct(path)
        .map_err(|e| AppError::Custom(format!("Failed to load: {}", e)))?;

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetAct125InfosCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_act160_get_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::Act160GetInfoCmd,
        Act160GetInfoReply,
        "activity160/get_info.json"
    );
    Ok(())
}

pub async fn on_act212_get_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let reply = GetAct212InfoReply {
        act212_info: Some(Act212InfoNo {
            activity_id: Some(13119),
            is_active: Some(false),
            bonuss: vec![Act212BonusNo {
                id: None,
                status: None,
            }],
            end_time: Some(0), // 2030-01-01 00:00:00 UTC
        }),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetAct212InfoCmd, reply, 0, req.up_tag)
        .await?;
    Ok(())
}

pub async fn on_act165_get_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::Act165GetInfoCmd,
        Act165GetInfoReply,
        "activity165/get_info.json"
    );
    Ok(())
}

pub async fn on_get_act208_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetAct208InfoCmd,
        GetAct208InfoReply,
        "activity208/get_info.json"
    );
    Ok(())
}

pub async fn on_get_act209_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    send_reply!(
        ctx,
        req.up_tag,
        CmdId::GetAct209InfoCmd,
        GetAct209InfoReply,
        "activity209/get_info.json"
    );
    Ok(())
}
