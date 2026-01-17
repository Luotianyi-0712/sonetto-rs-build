use crate::network::handler;
use crate::state::ConnectionContext;
use byteorder::{BE, ByteOrder};
use std::sync::Arc;
use tokio::{io::AsyncReadExt, sync::Mutex};

pub async fn handle_client(ctx: Arc<Mutex<ConnectionContext>>) -> anyhow::Result<()> {
    loop {
        let packet = {
            let conn = ctx.lock().await;
            let mut socket = conn.socket.lock().await;

            let mut header = [0u8; 4];
            if let Err(e) = socket.read_exact(&mut header).await {
                tracing::debug!("Client disconnected: {e}");
                return Ok(());
            }

            let packet_len = BE::read_i32(&header) as usize;
            let mut buffer = vec![0u8; packet_len];
            if let Err(e) = socket.read_exact(&mut buffer).await {
                tracing::warn!("Failed to read packet body ({} bytes): {e}", packet_len);
                return Ok(());
            }

            let mut packet = Vec::with_capacity(4 + packet_len);
            packet.extend_from_slice(&header);
            packet.extend_from_slice(&buffer);
            packet
        };

        if let Err(e) = handler::dispatch_command(ctx.clone(), &packet[..]).await {
            tracing::error!("Dispatch error: {e}");
            break;
        }

        {
            let mut conn = ctx.lock().await;
            if let Err(e) = conn.flush_send_queue().await {
                tracing::error!("Failed to flush send queue: {e}");
                break;
            }
        }
    }

    Ok(())
}
