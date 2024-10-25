use axum::{extract::State, routing::get, Json, Router};

use crate::AppState;

pub async fn api_routes() -> Router<AppState> {
    Router::new().route("/worlds", get(worlds))
}

async fn worlds(state: State<AppState>) -> Json<Vec<String>> {
    let worlds: Vec<String> = sqlx::query!("SELECT name FROM worlds;")
        .fetch_all(&state.db)
        .await
        .unwrap()
        .into_iter()
        .map(|record| record.name)
        .collect();

    Json(worlds)
}
