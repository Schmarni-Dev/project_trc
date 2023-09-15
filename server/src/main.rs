use anyhow::Result;
use backend::*;
use common::turtle_packets::InfoData;

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

// amount of times i dead locked myself in TOKIOOOOO: 1
#[tokio::main]
async fn main() -> Result<()> {
    // minutes wasted on trying to find an issue the it was just the logger being wongly configured: 10
    let _ = pretty_env_logger::formatted_timed_builder()
        .filter(None, log::LevelFilter::Debug)
        .init();
    let client_addr = "0.0.0.0:9001";
    let turtle_addr = "0.0.0.0:9002";

    let (turtle_connected_tx, turtle_connected_recv) =
        unbounded_channel::<(InfoData, WsSend, WsRecv)>();
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

    tokio::spawn(async{
        connection_manager::main(turtle_connected_recv, client_connected_recv)
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
