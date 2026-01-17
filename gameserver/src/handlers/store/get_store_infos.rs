use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use chrono::NaiveDateTime;
use prost::Message;
use sonettobuf::{CmdId, GetStoreInfosReply, GetStoreInfosRequest, GoodsInfo, StoreInfo};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_get_store_infos(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = GetStoreInfosRequest::decode(&req.data[..])?;
    tracing::info!("Received GetStoreInfosRequest: {:?}", request);

    let store_infos = {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;
        let pool = &conn.state.db;

        let game_data = data::exceldb::get();
        let mut store_infos = Vec::new();

        for store_id in &request.store_ids {
            let goods: Vec<_> = game_data
                .store_goods
                .iter()
                .filter(|g| g.store_id.parse::<i32>().unwrap_or(0) == *store_id)
                .filter(|g| g.is_online)
                .collect();

            let mut goods_infos = Vec::new();

            for good in goods {
                let buy_count: i32 = sqlx::query_scalar(
                    "SELECT buy_count FROM user_store_goods WHERE user_id = ? AND goods_id = ?",
                )
                .bind(player_id)
                .bind(good.id)
                .fetch_optional(pool)
                .await?
                .unwrap_or(0);

                let offline_time = if !good.offline_time.is_empty() {
                    NaiveDateTime::parse_from_str(&good.offline_time, "%Y-%m-%d %H:%M:%S")
                        .ok()
                        .map(|dt| dt.and_utc().timestamp_millis())
                        .unwrap_or(0)
                } else {
                    0
                };

                goods_infos.push(GoodsInfo {
                    goods_id: good.id,
                    buy_count,
                    offline_time: Some(offline_time),
                });
            }

            let next_refresh_time = 0;

            store_infos.push(StoreInfo {
                id: *store_id,
                next_refresh_time,
                goods_infos: goods_infos.clone(),
                offline_time: Some(0),
            });

            tracing::info!(
                "User {} loaded store {} with {} goods",
                player_id,
                store_id,
                goods_infos.len()
            );
        }

        store_infos
    };

    let data = GetStoreInfosReply { store_infos };

    {
        let mut conn = ctx.lock().await;
        conn.send_reply(CmdId::GetStoreInfosCmd, data, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
