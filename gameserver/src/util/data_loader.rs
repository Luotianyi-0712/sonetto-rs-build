use common::data_directory;
use std::fs;

/// Load and send a push message from JSON
/// Usage: send_push!(ctx, CmdId::HeroUpdatePushCmd, HeroUpdatePush, "login/hero_update.json");
#[macro_export]
macro_rules! send_push {
    ($ctx:expr, $cmd_id:expr, $msg_type:ty, $path:expr) => {{
        let msg: $msg_type = $crate::util::data_loader::GameDataLoader::load_struct($path)
            .map_err(|e| {
                $crate::error::AppError::Custom(format!(
                    "Failed to load {} from {}: {}",
                    stringify!($msg_type),
                    $path,
                    e
                ))
            })?;

        tracing::info!("Loaded message from {}: {:?}", $path, msg);

        let mut conn = $ctx.lock().await;
        conn.notify($cmd_id, msg).await?;
    }};
}

/// Load and send a reply message from JSON
/// Usage: send_reply!(ctx, req.up_tag, CmdId::GetPlayerInfoCmd, GetPlayerInfoReply, "players/default.json");
#[macro_export]
macro_rules! send_reply {
    ($ctx:expr, $up_tag:expr, $cmd_id:expr, $msg_type:ty, $path:expr) => {{
        let msg: $msg_type = $crate::util::data_loader::GameDataLoader::load_struct($path)
            .map_err(|e| {
                $crate::error::AppError::Custom(format!(
                    "Failed to load {} from {}: {}",
                    stringify!($msg_type),
                    $path,
                    e
                ))
            })?;

        let mut conn = $ctx.lock().await;
        conn.send_reply($cmd_id, msg, 0, $up_tag).await?;
    }};
}

/// Load a message struct from JSON
/// Usage: let player_info = load_message!(PlayerInfo, "players/default.json")?;
#[macro_export]
macro_rules! load_message {
    ($msg_type:ty, $path:expr) => {
        $crate::data_loader::GameDataLoader::load_struct::<$msg_type>($path).map_err(|e| {
            $crate::error::AppError::Custom(format!(
                "Failed to load {} from {}: {}",
                stringify!($msg_type),
                $path,
                e
            ))
        })
    };
}

#[allow(dead_code)]
pub struct GameDataLoader;

#[allow(dead_code)]
impl GameDataLoader {
    /// Load any message struct from JSON file
    pub fn load_struct<T>(relative_path: &str) -> Result<T, anyhow::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let file_path = data_directory().join(relative_path);

        if !file_path.exists() {
            return Err(anyhow::anyhow!(
                "Missing JSON file: {}\nPath: {:?}",
                relative_path,
                file_path
            ));
        }

        let json_data = fs::read_to_string(&file_path)?;
        let data: T = serde_json::from_str(&json_data)?;
        Ok(data)
    }
}
