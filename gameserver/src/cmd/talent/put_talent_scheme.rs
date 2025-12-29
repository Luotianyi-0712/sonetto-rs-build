use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::heroes;
use prost::Message;
use sonettobuf::{
    CmdId, HeroUpdatePush, PutTalentSchemeReply, PutTalentSchemeRequest, TalentCubeInfo,
    TalentTemplateInfo,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_put_talent_scheme(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = PutTalentSchemeRequest::decode(&req.data[..])?;
    tracing::info!("Received PutTalentSchemeRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
    let talent_id = request.talent_id.ok_or(AppError::InvalidRequest)?;
    let talent_mould = request.talent_mould.ok_or(AppError::InvalidRequest)?;
    let template_id = request.template_id.ok_or(AppError::InvalidRequest)?;

    let (user_id, template_info) = {
        let ctx_guard = ctx.lock().await;
        let player_id = ctx_guard.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &ctx_guard.state.db;

        let hero = heroes::get_hero_by_hero_id(pool, player_id, hero_id).await?;

        let game_data = data::exceldb::get();

        let talent_scheme = game_data
            .talent_scheme
            .iter()
            .find(|s| s.talent_id == talent_id && s.talent_mould == talent_mould);

        let talent_scheme = match talent_scheme {
            Some(s) => s,
            None => {
                tracing::error!(
                    "Talent scheme not found for talent {} mould {}",
                    talent_id,
                    talent_mould
                );
                return Err(AppError::InvalidRequest);
            }
        };

        let template_row_id: i64 = sqlx::query_scalar(
            "SELECT rowid FROM hero_talent_templates WHERE hero_uid = ? AND template_id = ?",
        )
        .bind(hero.record.uid)
        .bind(template_id)
        .fetch_one(pool)
        .await?;

        let cubes: Vec<(i32, i32, i32, i32)> = talent_scheme
            .talen_scheme
            .split('#')
            .filter_map(|cube_str| {
                let parts: Vec<&str> = cube_str.split(',').collect();
                if parts.len() == 4 {
                    let cube_id = parts[0].parse::<i32>().ok()?;
                    let direction = parts[1].parse::<i32>().ok()?;
                    let pos_x = parts[2].parse::<i32>().ok()?;
                    let pos_y = parts[3].parse::<i32>().ok()?;
                    Some((cube_id, direction, pos_x, pos_y))
                } else {
                    None
                }
            })
            .collect();

        sqlx::query("DELETE FROM hero_talent_template_cubes WHERE template_row_id = ?")
            .bind(template_row_id)
            .execute(pool)
            .await?;

        for (cube_id, direction, pos_x, pos_y) in &cubes {
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
        }

        tracing::info!(
            "Loaded {} cubes from talent scheme {} into template {}",
            cubes.len(),
            talent_id,
            template_id
        );

        if template_id == hero.record.use_talent_template_id {
            sqlx::query("DELETE FROM hero_talent_cubes WHERE hero_uid = ?")
                .bind(hero.record.uid)
                .execute(pool)
                .await?;

            for (cube_id, direction, pos_x, pos_y) in &cubes {
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
            talent_cube_infos,
            name: Some(template_data.0),
            style: Some(template_data.1),
        };

        (player_id, template_info)
    };

    let data = PutTalentSchemeReply {
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
            .send_reply(CmdId::PutTalentSchemeCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
