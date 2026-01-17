use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::heroes;
use prost::Message;
use sonettobuf::{CmdId, HeroUpdatePush, UnlockTalentStyleReply, UnlockTalentStyleRequest};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_unlock_talent_style(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = UnlockTalentStyleRequest::decode(&req.data[..])?;
    tracing::info!("Received UnlockTalentStyleRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let style = request.style.ok_or(AppError::InvalidRequest)?;

    let user_id = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        let hero = heroes::get_hero_by_hero_id(pool, player_id, hero_id).await?;

        let has_style: Option<i32> = sqlx::query_scalar(
            "SELECT 1 FROM hero_talent_styles WHERE hero_uid = ? AND style_id = ?",
        )
        .bind(hero.record.uid)
        .bind(style)
        .fetch_optional(pool)
        .await?;

        if has_style.is_some() {
            tracing::info!(
                "User {} already owns style {} for hero {}",
                player_id,
                style,
                hero_id
            );

            drop(conn);

            let mut conn = ctx.lock().await;
            let hero_proto: sonettobuf::HeroInfo = hero.into();
            conn.notify(
                CmdId::HeroHeroUpdatePushCmd,
                HeroUpdatePush {
                    hero_updates: vec![hero_proto],
                },
            )
            .await?;
            conn.send_reply(
                CmdId::UnlockTalentStyleCmd,
                UnlockTalentStyleReply {
                    hero_id: Some(hero_id),
                    style: Some(style),
                },
                0,
                req.up_tag,
            )
            .await?;

            return Ok(());
        }

        let game_data = data::exceldb::get();

        let style_cost = game_data
            .talent_style_cost
            .iter()
            .find(|s| s.hero_id == hero_id && s.style_id == style);

        let style_cost = match style_cost {
            Some(c) => c,
            None => {
                tracing::error!("Style cost not found for hero {} style {}", hero_id, style);
                return Err(AppError::InvalidRequest);
            }
        };

        let mut cost_items = Vec::new();
        let mut cost_currencies = Vec::new();

        for cost_part in style_cost.consume.split('|') {
            let parts: Vec<&str> = cost_part.split('#').collect();
            if parts.len() >= 3 {
                match parts[0] {
                    "1" => {
                        let item_id: u32 =
                            parts[1].parse().map_err(|_| AppError::InvalidRequest)?;
                        let amount: i32 = parts[2].parse().map_err(|_| AppError::InvalidRequest)?;
                        cost_items.push((item_id, amount));
                    }
                    "2" => {
                        let currency_id: i32 =
                            parts[1].parse().map_err(|_| AppError::InvalidRequest)?;
                        let amount: i32 = parts[2].parse().map_err(|_| AppError::InvalidRequest)?;
                        cost_currencies.push((currency_id, amount));
                    }
                    _ => {}
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

                drop(conn);

                crate::util::push::send_item_change_push(ctx.clone(), player_id, vec![*item_id])
                    .await?;

                let mut conn = ctx.lock().await;
                conn.send_reply(
                    CmdId::UnlockTalentStyleCmd,
                    UnlockTalentStyleReply {
                        hero_id: Some(hero_id),
                        style: Some(style),
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

                drop(conn);

                crate::util::push::send_currency_change_push(
                    ctx.clone(),
                    player_id,
                    vec![(*currency_id, 0)],
                )
                .await?;

                let mut conn = ctx.lock().await;
                conn.send_reply(
                    CmdId::UnlockTalentStyleCmd,
                    UnlockTalentStyleReply {
                        hero_id: Some(hero_id),
                        style: Some(style),
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

        sqlx::query("INSERT INTO hero_talent_styles (hero_uid, style_id) VALUES (?, ?)")
            .bind(hero.record.uid)
            .bind(style)
            .execute(pool)
            .await?;

        let style_bit = 1 << style;
        let new_unlock = hero.record.talent_style_unlock | style_bit;

        sqlx::query("UPDATE heroes SET talent_style_unlock = ? WHERE uid = ? AND user_id = ?")
            .bind(new_unlock)
            .bind(hero.record.uid)
            .bind(player_id)
            .execute(pool)
            .await?;

        tracing::info!(
            "User {} unlocked talent style {} for hero {}",
            player_id,
            style,
            hero_id
        );

        player_id
    };

    let data = UnlockTalentStyleReply {
        hero_id: Some(hero_id),
        style: Some(style),
    };

    {
        let mut conn = ctx.lock().await;

        let updated_hero = heroes::get_hero_by_hero_id(&conn.state.db, user_id, hero_id).await?;
        conn.notify(
            CmdId::HeroHeroUpdatePushCmd,
            HeroUpdatePush {
                hero_updates: vec![updated_hero.into()],
            },
        )
        .await?;

        conn.send_reply(CmdId::UnlockTalentStyleCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
