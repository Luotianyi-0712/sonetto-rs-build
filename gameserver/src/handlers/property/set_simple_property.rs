use crate::error::AppError;
use crate::network::packet::ClientPacket;
use crate::state::ConnectionContext;
use database::db::game::simple_property;
use prost::Message;
use sonettobuf::{
    CmdId, SetSimplePropertyReply, SetSimplePropertyRequest, SimpleProperty, SimplePropertyPush,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn on_set_simple_property(
    ctx: Arc<Mutex<ConnectionContext>>,
    req: ClientPacket,
) -> Result<(), AppError> {
    let request = SetSimplePropertyRequest::decode(&req.data[..])?;
    tracing::info!("Received SetSimplePropertyRequest: {:?}", request);

    let property_id = request.id.ok_or(AppError::InvalidRequest)?;
    let property_value = request.property.ok_or(AppError::InvalidRequest)?;

    {
        let conn = ctx.lock().await;
        let player_id = conn.player_id.ok_or(AppError::NotLoggedIn)?;

        simple_property::set_simple_property(
            &conn.state.db,
            player_id,
            property_id,
            property_value.clone(),
        )
        .await?;

        tracing::info!(
            "User {} set property {} to '{}'",
            player_id,
            property_id,
            property_value
        );
    }

    let reply = SetSimplePropertyReply {};

    {
        let mut conn = ctx.lock().await;

        // Send push notification for property update
        let push = SimplePropertyPush {
            simple_property: Some(SimpleProperty {
                id: Some(property_id),
                property: Some(property_value),
            }),
        };

        conn.notify(CmdId::SimplePropertyPushCmd, push).await?;

        conn.send_reply(CmdId::SetSimplePropertyCmd, reply, 0, req.up_tag)
            .await?;
    }

    Ok(())
}
