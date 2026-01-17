use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use crate::{error::AppError, send_push};
use database::db::game::charges;
use prost::Message;
#[allow(unused_imports)]
use sonettobuf::{
    CmdId, GainSpecialBlockPush, GetChargeInfoReply, GetMonthCardInfoReply, MaterialChangePush,
    MonthCardInfo, ReadChargeNewReply, ReadChargeNewRequest, UpdateRedDotPush,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_charge_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (charge_infos, sandbox) = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        let infos = charges::get_charge_infos(&conn.state.db, player_id).await?;
        let sandbox = charges::get_sandbox_settings(&conn.state.db, player_id).await?;

        (infos, sandbox)
    };

    let reply = GetChargeInfoReply {
        infos: charge_infos.into_iter().map(Into::into).collect(),
        sandbox_enable: Some(sandbox.sandbox_enable),
        sandbox_balance: Some(sandbox.sandbox_balance),
    };

    let mut conn = ctx.lock().await;
    conn.send_reply(CmdId::GetChargeInfoCmd, reply, 0, req.up_tag)
        .await?;

    Ok(())
}

pub async fn on_get_charge_push_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    {
        let mut conn = ctx.lock().await;
        conn.send_empty_reply(CmdId::GetChargePushInfoCmd, Vec::new(), 0, req.up_tag)
            .await?;
    }

    Ok(())
}

pub async fn on_get_month_card_info(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let (can_claim, current_time) = {
        let conn = ctx.lock().await;
        let current_time = common::time::ServerTime::now_ms();

        let can_claim = conn
            .player_state
            .as_ref()
            .map(|s| s.can_claim_month_card(current_time))
            .unwrap_or(false);

        (can_claim, current_time)
    };

    if can_claim {
        tracing::info!("Claiming month card bonus");

        // these send the birthday blocks bugged for now

        /*  send_push!(
            ctx,
            CmdId::GainSpecialBlockPushCmd,
            GainSpecialBlockPush,
            "charge/gain_special_block_push.json"
        );

        send_push!(
            ctx,
            CmdId::MaterialChangePushCmd,
            MaterialChangePush,
            "charge/material_change_push.json"
        );*/

        send_push!(
            ctx,
            CmdId::UpdateRedDotPushCmd,
            UpdateRedDotPush,
            "charge/update_red_dot_push.json"
        );

        // Update player state in one place and persist
        {
            let mut conn = ctx.lock().await;

            conn.update_and_save_player_state(|state| {
                state.claim_month_card(current_time);
                state.mark_activity_pushes_sent(current_time);
            })
            .await?;
        }
    } else {
        tracing::info!("Month card already claimed today");
    }

    // Send reply
    let reply = GetMonthCardInfoReply {
        infos: vec![MonthCardInfo {
            id: Some(610001),
            expire_time: Some(1767607200),
            has_get_bonus: Some(!can_claim),
        }],
    };

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::GetMonthCardInfoCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}

pub async fn on_read_charge_new(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = ReadChargeNewRequest::decode(&req.data[..])?;

    tracing::info!("Received ReadChargeNewRequest: {:?}", request);

    let data = ReadChargeNewReply {
        goods_ids: request.goods_ids,
    };

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::ReadChargeNewCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
