use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::heroes;
use prost::Message;
use sonettobuf::{CmdId, HeroTalentUpReply, HeroTalentUpRequest, HeroUpdatePush};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_hero_talent_up(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = HeroTalentUpRequest::decode(&req.data[..])?;
    tracing::info!("Received HeroTalentUpRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;

    let (user_id, new_talent_id) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &ctx_guard.state.db;

        let hero = heroes::get_hero_by_hero_id(pool, player_id, hero_id).await?;
        let current_talent = hero.record.talent;
        let new_talent = current_talent + 1;

        let game_data = data::exceldb::get();

        let talent_config = game_data
            .character_talent
            .iter()
            .find(|t| t.hero_id == hero_id && t.talent_id == new_talent);

        let talent_config = match talent_config {
            Some(t) => t,
            None => {
                tracing::info!("Hero {} already at max talent {}", hero_id, current_talent);

                let reply = HeroTalentUpReply {
                    hero_id: Some(hero_id),
                    talent_id: Some(current_talent),
                };

                drop(ctx_guard);

                let mut ctx_guard = ctx.lock().await;
                let hero_proto: sonettobuf::HeroInfo = hero.into();
                ctx_guard
                    .send_push(
                        CmdId::HeroHeroUpdatePushCmd,
                        HeroUpdatePush {
                            hero_updates: vec![hero_proto],
                        },
                    )
                    .await?;
                ctx_guard
                    .send_reply(CmdId::HeroTalentUpCmd, reply, 0, req.up_tag)
                    .await?;

                return Ok(());
            }
        };

        if hero.record.rank < talent_config.requirement {
            tracing::info!(
                "Hero {} rank {} does not meet talent {} requirement (needs rank {})",
                hero_id,
                hero.record.rank,
                new_talent,
                talent_config.requirement
            );

            let reply = HeroTalentUpReply {
                hero_id: Some(hero_id),
                talent_id: Some(current_talent),
            };

            drop(ctx_guard);

            let mut ctx_guard = ctx.lock().await;
            let hero_proto: sonettobuf::HeroInfo = hero.into();
            ctx_guard
                .send_push(
                    CmdId::HeroHeroUpdatePushCmd,
                    HeroUpdatePush {
                        hero_updates: vec![hero_proto],
                    },
                )
                .await?;
            ctx_guard
                .send_reply(CmdId::HeroTalentUpCmd, reply, 0, req.up_tag)
                .await?;

            return Ok(());
        }

        if !talent_config.consume.is_empty() {
            for cost_part in talent_config.consume.split('|') {
                let parts: Vec<&str> = cost_part.split('#').collect();
                if parts.len() >= 3 && parts[0] == "1" {
                    let item_id: u32 = parts[1].parse().map_err(|_| AppError::InvalidRequest)?;
                    let amount: i32 = parts[2].parse().map_err(|_| AppError::InvalidRequest)?;

                    let current = database::db::game::items::get_item(pool, player_id, item_id)
                        .await?
                        .map(|i| i.quantity)
                        .unwrap_or(0);

                    if current < amount {
                        tracing::info!(
                            "User {} insufficient item {} for talent up (has {}, needs {})",
                            player_id,
                            item_id,
                            current,
                            amount
                        );

                        drop(ctx_guard);

                        crate::utils::push::send_item_change_push(
                            ctx.clone(),
                            player_id,
                            vec![item_id],
                            vec![],
                            vec![],
                        )
                        .await?;

                        let mut ctx_guard = ctx.lock().await;
                        ctx_guard
                            .send_reply(
                                CmdId::HeroTalentUpCmd,
                                HeroTalentUpReply {
                                    hero_id: Some(hero_id),
                                    talent_id: Some(current_talent),
                                },
                                0,
                                req.up_tag,
                            )
                            .await?;

                        return Ok(());
                    }
                }
            }

            for cost_part in talent_config.consume.split('|') {
                let parts: Vec<&str> = cost_part.split('#').collect();
                if parts.len() >= 3 && parts[0] == "1" {
                    let item_id: u32 = parts[1].parse().unwrap();
                    let amount: i32 = parts[2].parse().unwrap();

                    database::db::game::items::remove_item_quantity(
                        pool, player_id, item_id, amount,
                    )
                    .await?;
                }
            }
        }

        sqlx::query("UPDATE heroes SET talent = ? WHERE uid = ? AND user_id = ?")
            .bind(new_talent)
            .bind(hero.record.uid)
            .bind(player_id)
            .execute(pool)
            .await?;

        tracing::info!(
            "User {} upgraded hero {} talent from {} to {}",
            player_id,
            hero_id,
            current_talent,
            new_talent
        );

        (player_id, new_talent)
    };

    let data = HeroTalentUpReply {
        hero_id: Some(hero_id),
        talent_id: Some(new_talent_id),
    };

    {
        let mut ctx_guard = ctx.lock().await;

        let updated_hero =
            heroes::get_hero_by_hero_id(&ctx_guard.state.db, user_id, hero_id).await?;
        ctx_guard
            .send_push(
                CmdId::HeroHeroUpdatePushCmd,
                HeroUpdatePush {
                    hero_updates: vec![updated_hero.into()],
                },
            )
            .await?;

        ctx_guard
            .send_reply(CmdId::HeroTalentUpCmd, data, 0, req.up_tag)
            .await?;

        tracing::info!("Hero {} talent upgraded to {}", hero_id, new_talent_id);
    }

    Ok(())
}
