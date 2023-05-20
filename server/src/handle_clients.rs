use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use log::info;
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};

use crate::data_types::server_turtle::{WsRecv, WsSend};

pub async fn handle_connection(
    raw_stream: TcpStream,
    addr: SocketAddr,
    client_connected: UnboundedSender<(WsSend, WsRecv)>,
) {
    info!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    info!("WebSocket connection established: {}", addr);

    // let (outgoing, incoming) = ;
    _ = client_connected.send(ws_stream.split());
}
