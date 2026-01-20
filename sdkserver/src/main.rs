use common::{config, excel_data_directory, host, http_port, init_config, init_tracing};
use database::{DatabaseSettings, connect_to, run_migrations};
use gameserver::state::AppState as GameState;
use reqwest::Client;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

mod handlers;
mod middleware;
mod models;

use middleware::crypto::sdk_encryption;
use middleware::logging::full_logger;

#[derive(Clone)]
pub struct SdkState {
    pub http_client: Client,
}

#[derive(Clone)]
pub struct AppState {
    pub sdk: SdkState,
    pub game: Arc<GameState>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config_path = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("config.toml")))
        .unwrap_or_else(|| PathBuf::from("config.toml"));

    let mut cfg = config::ServerConfig::load_or_create(&config_path)?;

    let config_dir = config_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    cfg.resolve_paths(&config_dir)?;
    cfg.validate_paths()?;

    info!("Server configuration:");
    info!("Host: {}:{}", cfg.server.host, cfg.server.http_port);

    init_config(cfg.clone());

    let db_settings = DatabaseSettings {
        db_name: config().database.path.to_string_lossy().to_string(),
        ..Default::default()
    };

    let db = connect_to(&db_settings).await?;
    run_migrations(&db).await?;

    info!("Loading game data...");
    codegen::configs::init(excel_data_directory().to_str().unwrap())?;
    info!("Game data loaded");

    let state = AppState {
        sdk: SdkState {
            http_client: Client::new(),
        },
        game: Arc::new(GameState::new(db)),
    };

    // Build router
    let with_encryption = handlers::router::account_router()
        .merge(handlers::router::trade_router())
        .layer(axum::middleware::from_fn(full_logger))
        .layer(axum::middleware::from_fn(sdk_encryption));

    let without_encryption = handlers::router::game_router()
        .merge(handlers::router::jsp_router())
        .merge(handlers::router::index_router())
        .layer(axum::middleware::from_fn(full_logger));

    let app = with_encryption.merge(without_encryption).with_state(state);

    let addr: SocketAddr = format!("{}:{}", host(), http_port()).parse()?;
    info!("HTTP Server listening on http://{}", addr);

    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
