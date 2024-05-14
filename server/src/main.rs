use std::{env, sync::Arc};

use anyhow::Result;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use backend::{db::DB, *};
use common::{
    extensions::Extensions,
    turtle_packets::{SetupInfoData, T2SPackets},
};

use futures_util::{
    pin_mut,
    stream::{SplitSink, SplitStream},
};

use log::info;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::unbounded_channel,
};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

pub const SUPPORTED_EXTENSIONS: &[Extensions] = &[Extensions::PositionTracking];

type WsSend = SplitSink<WebSocketStream<TcpStream>, Message>;
type WsRecv = SplitStream<WebSocketStream<TcpStream>>;

async fn get_worlds(State(db): State<Arc<DB>>) -> Json<Vec<String>> {
    let w = sqlx::query!("SELECT name FROM worlds;")
        .fetch_all(&*db)
        .await
        .unwrap()
        .into_iter()
        .map(|r| r.name)
        .collect::<Vec<_>>();

    // let w = db.get_worlds().await.unwrap();
    info!("{w:#?}");
    Json(w)
}

async fn add_world(State(db): State<Arc<DB>>, name: String) {
    sqlx::query!("INSERT OR IGNORE INTO worlds VALUES (?);", name)
        .execute(&*db)
        .await
        .unwrap();
    // db.create_world(&name).await.unwrap();
}

async fn get_supported_extensions() -> Json<Vec<&'static str>> {
    Json(
        SUPPORTED_EXTENSIONS
            .iter()
            .map(|e| e.string_ident())
            .collect(),
    )
}

// amount of times i dead locked myself in TOKIOOOOO: 1
#[tokio::main]
async fn main() -> Result<()> {
    // minutes wasted on trying to find an issue the it was just the logger being wongly configured: 10
    pretty_env_logger::formatted_timed_builder()
        .filter(None, log::LevelFilter::Warn)
        .filter(Some("backend"), log::LevelFilter::Debug)
        .init();
    // let db = Arc::new(SqliteConnection::.await?);
    let db = Arc::new(DB::connect(&env::var("DATABASE_URL")?).await?);
    let app = Router::new()
        .route("/get_worlds", get(get_worlds))
        .route("/get_supported_extensions", get(get_supported_extensions))
        .route("/add_world", post(add_world))
        .nest_service("/lua", tower_http::services::ServeDir::new("./lua"))
        .with_state(db.clone());
    let axum_listener = tokio::net::TcpListener::bind("0.0.0.0:9003").await?;
    tokio::spawn(async {
        axum::serve(axum_listener, app.into_make_service())
            .await
            .unwrap()
    });

    let client_addr = "0.0.0.0:9001";
    let turtle_addr = "0.0.0.0:9002";

    let (turtle_connected_tx, turtle_connected_recv) =
        unbounded_channel::<(SetupInfoData, Vec<T2SPackets>, WsSend, WsRecv)>();
    let (client_connected_tx, client_connected_recv) = unbounded_channel::<(WsSend, WsRecv)>();

    pin_mut!(turtle_connected_tx);
    pin_mut!(client_connected_tx);

    // Create the event loop and TCP listener we'll accept connections on.
    let client_listener = TcpListener::bind(&client_addr)
        .await
        .expect("Failed to bind Client Socket");
    info!("Client Socket Listening on: {}", client_addr);
    // Again
    let turtle_listener = TcpListener::bind(&turtle_addr)
        .await
        .expect("Failed to bind Trutle Socket");
    info!("Trutle Socket Listening on: {}", turtle_addr);

    // Let's spawn the handling of each connection in a separate task.
    let client_connected_tx = client_connected_tx.clone();
    tokio::spawn(async move {
        // info!("EXPLAIN!!!");
        let listener = client_listener;
        // This Is the Broken Listener!
        while let Ok((stream, addr)) = listener.accept().await {
            // info!("awdasd?!?!??!?!?!?!?");
            tokio::spawn(backend::handle_clients::handle_connection(
                stream,
                addr,
                client_connected_tx.clone(),
            ));
        }
    });

    let db_ = db.clone();
    tokio::spawn(async {
        connection_manager::main(turtle_connected_recv, client_connected_recv, db_)
            .await
            .unwrap();
    });

    while let Ok((stream, addr)) = turtle_listener.accept().await {
        // info!("dafuq?!");
        tokio::spawn(backend::handle_turtles::handle_connection(
            stream,
            addr,
            turtle_connected_tx.clone(),
        ));
    }
    Ok(())
    // loop {}
}
