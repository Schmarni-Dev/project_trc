use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};

use common::{
    turtle::{MoveDirection, TurnDir, Turtle},
    turtle_packets::{InfoData, T2SPackets},
    Pos3,
};

use futures_channel::mpsc::UnboundedSender;
use futures_util::{
    future,
    stream::{SplitSink, SplitStream},
    TryStreamExt,
};
#[allow(unused_imports)]
use log::info;
use serde_json::from_str;
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use crate::connection_manager::TurtleCommBus;

use super::arc_mutex::ArcMutex;

pub type WsSend = SplitSink<WebSocketStream<TcpStream>, Message>;
pub type WsRecv = SplitStream<WebSocketStream<TcpStream>>;

pub struct ServerTurtle {
    inner: Turtle,
    send: WsSend,
}
impl Deref for ServerTurtle {
    type Target = Turtle;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ServerTurtle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ServerTurtle {
    /// Check if inner exists allready in the db, if not make new Turtle else load Turtle from db!
    pub async fn new(
        inner: Turtle,
        send: WsSend,
        recv: WsRecv,
        comm_bus: UnboundedSender<TurtleCommBus>,
    ) -> ServerTurtle {
        let mut turtle = ServerTurtle { inner, send };
        turtle.init(recv);
        turtle
    }

    pub(crate) async fn on_msg_recived(&mut self, msg: T2SPackets) {
        match msg {
            T2SPackets::Info(InfoData {
                index: _,
                name,
                inventory,
                fuel,
                max_fuel,
            }) => {
                self.inner.fuel = fuel;
                self.inner.max_fuel = max_fuel;
                self.inner.inventory = inventory;
                self.inner.name = name;
                info!("Info Recived ^^7")
            }
            T2SPackets::Moved { direction } => {
                //TODO: Somehow Notify client of Change

                match direction {
                    MoveDirection::Forward => {
                        let forward = self.inner.get_forward_vec();
                        self.inner.position += forward;
                    }
                    MoveDirection::Back => {
                        let forward = self.inner.get_forward_vec();
                        self.inner.position -= forward;
                    }
                    MoveDirection::Up => self.inner.position += Pos3::new(0, 1, 0),
                    MoveDirection::Down => self.inner.position -= Pos3::new(0, 1, 0),
                    MoveDirection::Left => self.inner.orientation = self.inner.turn(TurnDir::Left),
                    MoveDirection::Right => {
                        self.inner.orientation = self.inner.turn(TurnDir::Right)
                    }
                }
            }
            T2SPackets::Blocks { up, down, front } => {
                info!("up: {:?}", up);
                info!("front: {:?}", front);
                info!("down: {:?}", down);
            }
        }
    }

    async fn init(&mut self, recv: WsRecv) {
        // V Actually yes Just the logging was FUCKED V
        // info!("am i ever inniting? .. no");

        recv.try_for_each(|msg| {
            if let Message::Text(msg) = msg {
                info!("recived msg: {}", msg);
                self.on_msg_recived(from_str::<T2SPackets>(&msg).unwrap());
            }
            future::ok(())
        })
        .await;
    }
}
