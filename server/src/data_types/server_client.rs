use common::client_packets::{C2SPackets, S2CPackets};

use futures::StreamExt;
use futures_channel::mpsc::UnboundedSender;
use futures_util::SinkExt;
use log::info;
use rand::prelude::*;

// use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tungstenite::Message;

use super::server_turtle::{WsRecv, WsSend};

#[derive(Clone, Debug)]
pub enum ClientComms {
    Packet(C2SPackets),
    #[allow(non_camel_case_types)]
    KILL_ME,
}

pub struct ServerClient {
    ws_send: WsSend,
    msg_send: UnboundedSender<(i32, ClientComms)>,
    index: i32,
}

impl ServerClient {
    // TODO: add Channel for Turtle Comms
    pub fn new(
        ws_recv: WsRecv,
        ws_send: WsSend,
        msg_send: UnboundedSender<(i32, ClientComms)>,
    ) -> ServerClient {
        let mut s = ServerClient {
            ws_send,
            msg_send,
            index: random(),
        };
        s.init(ws_recv);
        s
    }

    fn init(&mut self, mut ws_recv: WsRecv) {
        let mut send = self.msg_send.clone();
        let index = self.get_index();
        tokio::spawn(async move {
            while let Some(data) = ws_recv.next().await {
                match data {
                    Err(_) | Ok(Message::Close(_)) => {
                        _ = send.send((index, ClientComms::KILL_ME)).await;
                    }
                    Ok(Message::Text(msg)) => {
                        if let Ok(msg) = serde_json::from_str::<C2SPackets>(&msg) {
                            match msg {
                                packet => {
                                    send.send((index, ClientComms::Packet(packet)))
                                        .await
                                        .unwrap();
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }
    pub async fn send_msg(&mut self, msg: &S2CPackets) {
        info!("Sending: {}", serde_json::to_string_pretty(msg).unwrap());
        if self
            .ws_send
            .send(Message::Text(serde_json::to_string(msg).unwrap()))
            .await
            .is_err()
        {
            info!("Welp Shit... KILL ME!!!");
            _ = self.msg_send.send((self.get_index(), ClientComms::KILL_ME));
        }
    }
    pub fn get_index(&self) -> i32 {
        self.index
    }
    pub async fn delete(mut self) {
        self.msg_send.close().await.unwrap();
        self.ws_send.close().await.unwrap();
        drop(self);
    }
}
