use data::exceldb;
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};

/// Load minimal critter info (one starter)
pub async fn load_critter_info(tx: &mut Transaction<'_, Sqlite>, uid: i64) -> sqlx::Result<()> {
    let now = common::time::ServerTime::now_ms();
    let critter_uid = 10000000i64;
    sqlx::query(
        r#"
        INSERT INTO critters (
            uid, player_id, define_id, create_time,
            efficiency, patience, lucky,
            efficiency_incr_rate, patience_incr_rate, lucky_incr_rate,
            special_skin, current_mood, is_locked, finish_train, is_high_quality,
            train_hero_id, total_finish_count, name,
            created_at, updated_at
        ) VALUES (?, ?, 1, ?, 100, 100, 100, 10, 10, 10, false, 50, false, false, false, 0, 0, 'Starter Critter', ?, ?)
        "#,
    )
    .bind(critter_uid)
    .bind(uid)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Load basic player info + show heroes
pub async fn load_player_info(tx: &mut Transaction<'_, Sqlite>, uid: i64) -> sqlx::Result<()> {
    let now = common::time::ServerTime::now_ms();
    sqlx::query(
        "INSERT INTO player_info (
            player_id, signature, birthday, portrait, show_achievement, bg,
            total_login_days, last_episode_id, last_logout_time,
            hero_rare_nn_count, hero_rare_n_count, hero_rare_r_count,
            hero_rare_sr_count, hero_rare_ssr_count,
            created_at, updated_at
        ) VALUES (?, 'Welcome to Sonetto!', '', 171603, '', 0, 1, 0, NULL, 1, 1, 1, 0, 0, ?, ?)",
    )
    .bind(uid)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    // Default show heroes (3 as in original)
    let default_heroes = [
        (3086, 180, 4, 5, 308603),
        (3120, 180, 4, 5, 312002),
        (3095, 180, 4, 5, 309502),
    ];
    for (i, (hero_id, level, rank, ex_skill_level, skin)) in default_heroes.iter().enumerate() {
        sqlx::query(
            "INSERT INTO player_show_heroes
             (player_id, hero_id, level, rank, ex_skill_level, skin, display_order)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(uid)
        .bind(hero_id)
        .bind(level)
        .bind(rank)
        .bind(ex_skill_level)
        .bind(skin)
        .bind(i as i32)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

/// Load user stats (fixes the "User stats not found" error)
pub async fn load_starter_user_stats(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO user_stats (user_id, first_charge, total_charge_amount, is_first_login, user_tag)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(user_id)
    .bind(false)
    .bind(0)
    .bind(true)
    .bind("用户类型7")
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Load minimal heroes (3 starters + basic related data)
pub async fn load_hero_list(
    tx: &mut Transaction<'_, Sqlite>,
    uid: i64,
    equip_map: &HashMap<i32, i64>,
) -> sqlx::Result<()> {
    let game_data = exceldb::get();
    let now = common::time::ServerTime::now_ms();
    static HERO_UID_COUNTER: AtomicI64 = AtomicI64::new(20000000);

    let starter_ids = [3086, 3120, 3095];
    for hero_id in starter_ids {
        let hero_uid = HERO_UID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let character = game_data
            .character
            .iter()
            .find(|c| c.id == hero_id)
            .cloned()
            .expect("Starter hero missing");

        let level = 180;
        let rank = 4;
        let skin = character.skin_id;
        let equip_id = character
            .equip_rec
            .split('#')
            .next()
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1501);
        let default_equip_uid = *equip_map.get(&equip_id).unwrap_or(&0);

        sqlx::query(
            r#"
            INSERT INTO heroes (
                uid, user_id, hero_id, create_time, level, exp, rank, breakthrough, skin, faith,
                active_skill_level, ex_skill_level, is_new, talent, default_equip_uid, duplicate_count,
                use_talent_template_id, talent_style_unlock, talent_style_red, is_favor,
                destiny_rank, destiny_level, destiny_stone, red_dot, extra_str,
                base_hp, base_attack, base_defense, base_mdefense, base_technic,
                base_multi_hp_idx, base_multi_hp_num,
                ex_cri, ex_recri, ex_cri_dmg, ex_cri_def, ex_add_dmg, ex_drop_dmg
            ) VALUES (?, ?, ?, ?, ?, 0, ?, 0, ?, 10400, 1, 5, false, 1, ?, 5, 1, 1, 0, false, 0, 0, 0, 0, '',
                5000, 500, 300, 300, 100, 0, 0, 50, 50, 1500, 0, 0, 0)
            "#,
        )
        .bind(hero_uid)
        .bind(uid)
        .bind(hero_id)
        .bind(now)
        .bind(level)
        .bind(rank)
        .bind(skin)
        .bind(default_equip_uid)
        .execute(&mut **tx)
        .await?;

        // Basic birthday info
        sqlx::query(
            "INSERT INTO hero_birthday_info (user_id, hero_id, birthday_count) VALUES (?, ?, 1)",
        )
        .bind(uid)
        .bind(hero_id)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

/// Load some equipment (enough for starters)
pub async fn load_equipment(
    tx: &mut Transaction<'_, Sqlite>,
    uid: i64,
) -> sqlx::Result<HashMap<i32, i64>> {
    let now = common::time::ServerTime::now_ms();
    static EQUIP_UID_COUNTER: AtomicI64 = AtomicI64::new(30000000);
    let mut equip_map: HashMap<i32, i64> = HashMap::new();

    let starter_equips = [1501, 1502, 1503, 1527, 1530]; // A few basics
    for equip_id in starter_equips {
        let equip_uid = EQUIP_UID_COUNTER.fetch_add(1, Ordering::SeqCst);
        sqlx::query(
            r#"
            INSERT INTO equipment (
                uid, user_id, equip_id, level, exp, break_lv, count, is_lock, refine_lv, created_at, updated_at
            ) VALUES (?, ?, ?, 60, 0, 3, 1, false, 5, ?, ?)
            "#,
        )
        .bind(equip_uid)
        .bind(uid)
        .bind(equip_id)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await?;
        equip_map.insert(equip_id, equip_uid);
    }
    Ok(equip_map)
}

/// Load essential currencies + some items
pub async fn load_starter_items(
    tx: &mut Transaction<'_, Sqlite>,
    user_id: i64,
) -> sqlx::Result<()> {
    let now = common::time::ServerTime::now_ms();

    // Generous starter currencies
    let currencies = [
        (1, 100),     // Common
        (2, 3000000), // Dust
        (3, 3000000), // Sharpodonty
        (4, 240),     // Energy
        (5, 3000000), // Another key resource
    ];
    for (id, qty) in currencies {
        sqlx::query(
            "INSERT INTO currencies (user_id, currency_id, quantity, last_recover_time, expired_time)
             VALUES (?, ?, ?, ?, 0)"
        )
        .bind(user_id)
        .bind(id)
        .bind(qty)
        .bind(now)
        .execute(&mut **tx)
        .await?;
    }

    // A few basic items
    let basic_items = [110101, 110201, 481002]; // Materials/selectors
    for item_id in basic_items {
        sqlx::query(
            "INSERT INTO items (user_id, item_id, quantity, last_use_time, last_update_time, total_gain_count)
             VALUES (?, ?, 10, NULL, ?, 10)"
        )
        .bind(user_id)
        .bind(item_id)
        .bind(now)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

/// Minimal all loader (added user_stats and more essentials)
pub async fn load_all_starter_data(pool: &SqlitePool, uid: i64) -> sqlx::Result<()> {
    tracing::info!(
        "Loading reduced starter data for uid {uid} (more than minimal to avoid crashes)"
    );
    let mut tx = pool.begin().await?;

    load_player_info(&mut tx, uid).await?;
    load_starter_user_stats(&mut tx, uid).await?;
    load_critter_info(&mut tx, uid).await?;
    let equip_map = load_equipment(&mut tx, uid).await?;
    load_hero_list(&mut tx, uid, &equip_map).await?;
    load_starter_items(&mut tx, uid).await?;

    // Add a few more common essentials to prevent early crashes
    // Basic guides (mark as completed)
    sqlx::query("INSERT INTO guide_progress (user_id, guide_id, step_id) VALUES (?, 1, -1)")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    // Basic summon stats
    sqlx::query(
        "INSERT INTO user_summon_stats (user_id, free_equip_summon, is_show_new_summon, new_summon_count, total_summon_count)
         VALUES (?, false, false, 0, 0)"
    )
    .bind(uid)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    tracing::info!("Finished loading reduced starter data for uid {uid}");
    Ok(())
}
