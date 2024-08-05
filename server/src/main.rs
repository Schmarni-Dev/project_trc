pub mod clients;
pub mod http_api;
pub mod session;
pub mod turtles;

use std::{env, sync::Arc};

use axum::{
    extract::ws::{Message, WebSocket},
    routing::get,
    Router,
};
use clients::{client_ws_route, ClientWsSender};
use color_eyre::eyre::Ok;
use futures::stream::SplitSink;
use http_api::api_routes;
use log::info;
use serde::Serialize;
use session::{SessionMap, Sessions};
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use turtles::{turtle_ws_route, TurtleWsSender};

pub struct AppStateData {
    db: SqlitePool,
    client_senders: RwLock<SessionMap<ClientWsSender>>,
    client_current_world: RwLock<SessionMap<Option<String>>>,
    sessions: Sessions,
    turtle_senders: RwLock<SessionMap<TurtleWsSender>>,
    turtle_i_w_map: RwLock<SessionMap<(i32, String)>>,
}

pub type AppData = Arc<AppStateData>;

#[derive(Serialize)]
pub struct Cool(String);

#[tokio::main]
pub async fn main() -> color_eyre::Result<()> {
    pretty_env_logger::formatted_timed_builder()
        .filter(None, log::LevelFilter::Warn)
        .filter(Some("trc_server"), log::LevelFilter::Debug)
        .init();

    info!(
        "{}",
        serde_json::to_string_pretty(&Cool("WOW".to_string())).unwrap()
    );
    info!(
        "{}",
        serde_json::to_string_pretty(&("SUS".to_string(), 69.0)).unwrap()
    );

    // let state: AppData = Arc::new(AppStateData {
    //     db: SqlitePool::connect(&env::var("DATABASE_URL")?).await?,
    //     client_senders: RwLock::default(),
    //     client_current_world: RwLock::default(),
    //     sessions: Sessions::default(),
    //     turtle_senders: RwLock::default(),
    //     turtle_i_w_map: RwLock::default(),
    // });
    //
    // let app = Router::new()
    //     .route("/", get(|| async { "Hello, World!" }))
    //     .route("/turtle/ws", get(turtle_ws_route))
    //     .route("/client/ws", get(client_ws_route))
    //     .nest("/api", api_routes().await)
    //     .nest_service("/turtle/lua", tower_http::services::ServeDir::new("./lua"))
    //     .with_state(state);
    //
    // // run our app with hyper, listening globally on port 8080
    // let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    // axum::serve(listener, app).await?;
    Ok(())
}
