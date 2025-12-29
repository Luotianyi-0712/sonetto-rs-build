use crate::error::AppError;
use crate::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::heroes;
use prost::Message;
use sonettobuf::{
    CmdId, HeroUpdatePush, TalentCubeInfo, TalentTemplateInfo, UseTalentTemplateReply,
    UseTalentTemplateRequest,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_use_talent_template(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = UseTalentTemplateRequest::decode(&req.data[..])?;
    tracing::info!("Received UseTalentTemplateRequest: {:?}", request);

    let hero_id = request.hero_id.ok_or(AppError::InvalidRequest)?;
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

        let cubes: Vec<(i32, i32, i32, i32)> = sqlx::query_as(
            "SELECT cube_id, direction, pos_x, pos_y
             FROM hero_talent_template_cubes
             WHERE template_row_id = ?",
        )
        .bind(template_row_id)
        .fetch_all(pool)
        .await?;

        let template_data: (String, i32) =
            sqlx::query_as("SELECT name, style FROM hero_talent_templates WHERE id = ?")
                .bind(template_row_id)
                .fetch_one(pool)
                .await?;

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

        sqlx::query("UPDATE heroes SET use_talent_template_id = ? WHERE uid = ? AND user_id = ?")
            .bind(template_id)
            .bind(hero.record.uid)
            .bind(player_id)
            .execute(pool)
            .await?;

        tracing::info!(
            "User {} switched to talent template {} for hero {}",
            player_id,
            template_id,
            hero_id
        );

        let talent_cube_infos = cubes
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

    let data = UseTalentTemplateReply {
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
            .send_reply(CmdId::UseTalentTemplateCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
