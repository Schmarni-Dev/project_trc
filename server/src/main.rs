use std::sync::Arc;

use anyhow::Result;
use backend::*;
use common::{turtle::TurtleIndexType, turtle_packets::InfoData};

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

type WsSend = SplitSink<WebSocketStream<TcpStream>, Message>;
type WsRecv = SplitStream<WebSocketStream<TcpStream>>;

#[tokio::main]
async fn main() -> Result<()> {
    // minutes wasted on trying to find an issue the it was just the logger being wongly configured: 10
    let _ = pretty_env_logger::formatted_builder()
        .filter(None, log::LevelFilter::Debug)
        .init();
    let client_addr = "0.0.0.0:9001";
    let turtle_addr = "0.0.0.0:9002";

    let (turtle_connected_tx, turtle_connected_recv) =
        unbounded_channel::<(InfoData, WsSend, WsRecv)>();
    // FIXME: Arc Mutex Channel? WTF!!!!
    let turtle_connected_send = Arc::new(tokio::sync::Mutex::new(turtle_connected_tx));
    pin_mut!(turtle_connected_send);

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
    tokio::spawn(async {
        let listener = client_listener;
        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(backend::handle_clients::handle_connection(stream, addr));
        }
    });

    tokio::spawn(connection_manager::main(turtle_connected_recv));

    while let Ok((stream, addr)) = turtle_listener.accept().await {
        // info!("dafuq?!");
        let turtle_connected_send = turtle_connected_send.clone();
        tokio::spawn(backend::handle_turtles::handle_connection(
            stream,
            addr,
            turtle_connected_send,
        ));
    }

    Ok(())
}
