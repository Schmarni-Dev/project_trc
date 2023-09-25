use common::client_packets::{C2SPackets, S2CPackets};

use futures::StreamExt;
use futures_channel::mpsc::UnboundedSender;
use futures_util::SinkExt;
use log::{error, info};
use rand::prelude::*;

use tokio::task::JoinHandle;
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
    ws_read_handle: Option<JoinHandle<()>>,
    index: i32,
    chunk_render_distance: u8,
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
            ws_read_handle: None,
            chunk_render_distance: 8,
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
                                packet => _ = send.send((index, ClientComms::Packet(packet))).await,
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }
    pub async fn send_msg(&mut self, msg: &S2CPackets) {
        let text = serde_json::to_string(msg).unwrap();
        if let Err(err) = self
            .ws_send
            .send(Message::Text(text))
            .await
        {
            error!("Error When sending Shit to Client: {}", err);
            _ = self.msg_send.send((self.get_index(), ClientComms::KILL_ME));
        }
    }
    pub fn get_index(&self) -> i32 {
        self.index
    }
    pub async fn delete(mut self) {
        _=self.msg_send.close().await;
        _=self.ws_send.close().await;
        if let Some(h) = self.ws_read_handle {
            h.abort();
        }
    }
}
