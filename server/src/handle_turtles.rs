use std::{net::SocketAddr, time::Duration};

use common::turtle_packets::{InfoData, S2TPackets, T2SPackets};
use futures::{SinkExt, StreamExt};
use log::info;
use serde_json::{from_str, to_string_pretty};
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};
use tungstenite::Message;

use crate::data_types::server_turtle::{WsRecv, WsSend};

pub async fn handle_connection(
    raw_stream: TcpStream,
    addr: SocketAddr,
    turtle_connected_send: UnboundedSender<(InfoData, WsSend, WsRecv)>,
) {
    if addr.to_string().contains("35.177.97.185") {
        drop(raw_stream);
        return;
    }
    info!("Incoming TCP connection from: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    info!("WebSocket connection established");
    let (mut outgoing, mut incoming) = ws_stream.split();
    outgoing
        .send(tungstenite::Message::Text(
            to_string_pretty(&S2TPackets::GetInfo).unwrap(),
        ))
        .await
        .unwrap();
    while let Some(Ok(msg)) = incoming.next().await {
        if let Message::Text(msg) = msg {
            if let T2SPackets::Info(data) = from_str::<T2SPackets>(&msg).unwrap() {
                turtle_connected_send
                    .send((data, outgoing, incoming))
                    .unwrap();
                break;
            }
        };
    }
}
