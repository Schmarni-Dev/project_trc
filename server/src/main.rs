pub mod clients;
pub mod computers;
pub mod http_api;
pub mod session;
pub mod turtles;

use std::{env, sync::Arc};

use axum::{routing::get, Router};
use bevy_ecs::world::World;
use clients::client_ws_route;
use color_eyre::eyre::Ok;
use derive_more::{Deref, DerefMut};
use http_api::api_routes;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use turtles::turtle_ws_route;

#[derive(Deref, DerefMut)]
pub struct InnerAppState {
    pub db: SqlitePool,
    #[deref]
    #[deref_mut]
    pub world: RwLock<World>,
}
#[derive(Deref, DerefMut, Clone)]
pub struct AppState(Arc<InnerAppState>);

#[tokio::main]
pub async fn main() -> color_eyre::Result<()> {
    pretty_env_logger::formatted_timed_builder()
        .filter(None, log::LevelFilter::Warn)
        .filter(Some("trc_server"), log::LevelFilter::Debug)
        .init();

    let state = AppState(Arc::new(InnerAppState {
        db: SqlitePool::connect(&env::var("DATABASE_URL")?).await?,
        world: RwLock::new(World::new()),
    }));

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/turtle/ws", get(turtle_ws_route))
        .route("/client/ws", get(client_ws_route))
        .nest("/api", api_routes().await)
        .nest_service("/turtle/lua", tower_http::services::ServeDir::new("./lua"))
        .with_state(state);

    // run our app with hyper, listening globally on port 8080
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
