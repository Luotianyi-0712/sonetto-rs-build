use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::heroes;
use prost::Message;
use sonettobuf::{
    CmdId, HeroUpdatePush, PutTalentCubeReply, PutTalentCubeRequest, TalentCubeInfo,
    TalentTemplateInfo,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_put_talent_cube(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = PutTalentCubeRequest::decode(&req.data[..])?;
    tracing::info!("Received PutTalentCubeRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let get_cube_info = request.get_cube_info;
    let put_cube_info = request.put_cube_info;
    let template_id = request.template_id.ok_or(AppError::InvalidRequest)?;

    let (user_id, template_info) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &ctx_guard.state.db;

        let hero = heroes::get_hero_by_hero_id(pool, player_id, hero_id).await?;

        let template_row_id: i64 = sqlx::query_scalar(
            "SELECT id FROM hero_talent_templates WHERE hero_uid = ? AND template_id = ?",
        )
        .bind(hero.record.uid)
        .bind(template_id)
        .fetch_one(pool)
        .await?;

        if let Some(get_cube) = &get_cube_info {
            sqlx::query(
                "DELETE FROM hero_talent_template_cubes
                 WHERE template_row_id = ? AND pos_x = ? AND pos_y = ?",
            )
            .bind(template_row_id)
            .bind(get_cube.pos_x.unwrap_or(0))
            .bind(get_cube.pos_y.unwrap_or(0))
            .execute(pool)
            .await?;

            tracing::info!(
                "Removed cube from template {} at ({}, {})",
                template_id,
                get_cube.pos_x.unwrap_or(0),
                get_cube.pos_y.unwrap_or(0)
            );
        }

        if let Some(put_cube) = &put_cube_info {
            let cube_id = put_cube.cube_id.ok_or(AppError::InvalidRequest)?;
            let direction = put_cube.direction.ok_or(AppError::InvalidRequest)?;
            let pos_x = put_cube.pos_x.ok_or(AppError::InvalidRequest)?;
            let pos_y = put_cube.pos_y.ok_or(AppError::InvalidRequest)?;

            sqlx::query(
                "DELETE FROM hero_talent_template_cubes
                 WHERE template_row_id = ? AND pos_x = ? AND pos_y = ?",
            )
            .bind(template_row_id)
            .bind(pos_x)
            .bind(pos_y)
            .execute(pool)
            .await?;

            // Insert new cube
            sqlx::query(
                "INSERT INTO hero_talent_template_cubes
                 (template_row_id, cube_id, direction, pos_x, pos_y)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(template_row_id)
            .bind(cube_id)
            .bind(direction)
            .bind(pos_x)
            .bind(pos_y)
            .execute(pool)
            .await?;

            tracing::info!(
                "Placed cube {} at ({}, {}) in template {}",
                cube_id,
                pos_x,
                pos_y,
                template_id
            );
        }

        if template_id == hero.record.use_talent_template_id {
            if let Some(get_cube) = &get_cube_info {
                sqlx::query(
                    "DELETE FROM hero_talent_cubes
                     WHERE hero_uid = ? AND pos_x = ? AND pos_y = ?",
                )
                .bind(hero.record.uid)
                .bind(get_cube.pos_x.unwrap_or(0))
                .bind(get_cube.pos_y.unwrap_or(0))
                .execute(pool)
                .await?;
            }

            if let Some(put_cube) = &put_cube_info {
                let cube_id = put_cube.cube_id.unwrap();
                let direction = put_cube.direction.unwrap();
                let pos_x = put_cube.pos_x.unwrap();
                let pos_y = put_cube.pos_y.unwrap();

                sqlx::query(
                    "DELETE FROM hero_talent_cubes
                     WHERE hero_uid = ? AND pos_x = ? AND pos_y = ?",
                )
                .bind(hero.record.uid)
                .bind(pos_x)
                .bind(pos_y)
                .execute(pool)
                .await?;

                sqlx::query(
                    "INSERT INTO hero_talent_cubes
                     (hero_uid, cube_id, direction, pos_x, pos_y)
                     VALUES (?, ?, ?, ?, ?)",
                )
                .bind(hero.record.uid)
                .bind(cube_id)
                .bind(direction)
                .bind(pos_x)
                .bind(pos_y)
                .execute(pool)
                .await?;
            }

            tracing::info!(
                "Updated active talent cubes for hero {} (template {})",
                hero_id,
                template_id
            );
        }

        let template_data: (String, i32) =
            sqlx::query_as("SELECT name, style FROM hero_talent_templates WHERE id = ?")
                .bind(template_row_id)
                .fetch_one(pool)
                .await?;

        let cubes: Vec<(i32, i32, i32, i32)> = sqlx::query_as(
            "SELECT cube_id, direction, pos_x, pos_y
             FROM hero_talent_template_cubes
             WHERE template_row_id = ?",
        )
        .bind(template_row_id)
        .fetch_all(pool)
        .await?;

        let talent_cube_infos: Vec<TalentCubeInfo> = cubes
            .into_iter()
            .map(|(cube_id, direction, pos_x, pos_y)| TalentCubeInfo {
                cube_id: Some(cube_id),
                direction: Some(direction),
                pos_x: Some(pos_x),
                pos_y: Some(pos_y),
            })
            .collect();

        let template_info = TalentTemplateInfo {
            id: Some(template_id),
            talent_cube_infos: talent_cube_infos,
            name: Some(template_data.0),
            style: Some(template_data.1),
        };

        (player_id, template_info)
    };

    let data = PutTalentCubeReply {
        hero_id: Some(hero_id),
        template_info: Some(template_info),
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
            .send_reply(CmdId::PutTalentCubeCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
