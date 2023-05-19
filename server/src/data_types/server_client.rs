use common::client_packets::{C2SPackets, S2CPackets};

use futures::StreamExt;
use futures_channel::mpsc::UnboundedSender;
use futures_util::SinkExt;
use rand::prelude::*;

// use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tungstenite::Message;

use super::server_turtle::{WsRecv, WsSend};

pub enum MsgType {
    Single(),
}

pub struct ServerClient {
    ws_send: WsSend,
    msg_send: UnboundedSender<(i32, C2SPackets)>,
    index: i32,
}

impl ServerClient {
    // TODO: add Channel for Turtle Comms
    pub fn new(
        ws_recv: WsRecv,
        ws_send: WsSend,
        msg_send: UnboundedSender<(i32, C2SPackets)>,
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
            while let Some(Ok(msg)) = ws_recv.next().await {
                if let Message::Text(msg) = msg {
                    if let Ok(msg) = serde_json::from_str::<C2SPackets>(&msg) {
                        match msg {
                            C2SPackets::MoveTurtle { .. } => {
                                let _ = send.send((index, msg)).await;
                            }
                            _ => {}
                        }
                    }
                };
            }
        });
    }
    pub async fn send_msg(&mut self, msg: &S2CPackets) {
        self.ws_send
            .send(Message::Text(serde_json::to_string_pretty(msg).unwrap()))
            .await
            .unwrap();
    }
    pub fn get_index(&self) -> i32 {
        self.index
    }
}
