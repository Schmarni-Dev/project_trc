use axum::{extract::State, routing::get, Json, Router};

use crate::AppData;

pub async fn api_routes() -> Router<AppData> {
    Router::new().route("/worlds", get(worlds))
}

async fn worlds(state: State<AppData>) -> Json<Vec<String>> {
    let worlds: Vec<String> = sqlx::query!("SELECT name FROM worlds;")
        .fetch_all(&state.db)
        .await
        .unwrap()
        .into_iter()
        .map(|record| record.name)
        .collect();

    Json(worlds)
}
