use std::net::SocketAddr;

use common::turtle_packets::{S2TPackets, SetupInfoData, T2SPackets};
use futures::{SinkExt, StreamExt};
use log::info;
use serde_json::{from_str, to_string_pretty};
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};
use tungstenite::Message;

use crate::data_types::server_turtle::{WsRecv, WsSend};

pub async fn handle_connection(
    raw_stream: TcpStream,
    addr: SocketAddr,
    turtle_connected_send: UnboundedSender<(SetupInfoData, Vec<T2SPackets>, WsSend, WsRecv)>,
) -> anyhow::Result<()> {
    if addr.to_string().contains("35.177.97.185") {
        drop(raw_stream);
        return Ok(());
    }
    info!("Incoming TCP connection from: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(raw_stream).await?;
    // .expect("Error during the websocket handshake occurred");
    info!("WebSocket connection established");
    let (mut outgoing, mut incoming) = ws_stream.split();
    outgoing
        .send(tungstenite::Message::Text(
            to_string_pretty(&S2TPackets::GetSetupInfo).unwrap(),
        ))
        .await
        .unwrap();
    while let Some(Ok(msg)) = incoming.next().await {
        if let Message::Text(msg) = msg {
            match from_str::<T2SPackets>(&msg).unwrap() {
                T2SPackets::Batch(data) => {
                    let info = match data.as_slice() {
                        [T2SPackets::SetupInfo(i), ..] => i.clone(),
                        _ => {
                            info!("invalid Setup Packet!");
                            outgoing.close().await?;
                            break;
                        }
                    };
                    turtle_connected_send.send((info, data, outgoing, incoming))?;
                    break;
                }
                T2SPackets::Ping => {}
                _ => {
                    info!("invalid Setup Packet!");
                    outgoing.close().await?;
                    break;
                }
            }
        } else {
            info!("invalid Setup Packet!");
            outgoing.close().await?;
            break;
        };
    }
    Ok(())
}
