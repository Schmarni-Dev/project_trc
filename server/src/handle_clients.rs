use std::net::SocketAddr;

use futures_channel::mpsc::unbounded;
use futures_util::StreamExt;
use log::info;
use tokio::net::TcpStream;

use crate::data_types::{client_map::ClientMap, server_client::ServerClient};

pub async fn handle_connection(raw_stream: TcpStream, addr: SocketAddr) {
    info!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    info!("WebSocket connection established: {}", addr);

    let (outgoing, incoming) = ws_stream.split();
    let (tx, _) = unbounded();
    let client = ServerClient::new(incoming, outgoing, tx);
    let mut map = ClientMap::new();
    map.push(client);
    map.broadcast(common::client_packets::S2CPackets::RequestedTurtles(
        Vec::new(),
    ))
    .await;
}
