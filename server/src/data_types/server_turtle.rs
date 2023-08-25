use std::ops::{Deref, DerefMut};

use common::{
    turtle::{MoveDirection, TurnDir, Turtle},
    turtle_packets::{InfoData, S2TPackets, T2SPackets},
    world_data::Block,
    Pos3,
};

use futures_channel::mpsc::UnboundedSender;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
#[allow(unused_imports)]
use log::info;
use serde_json::{from_str, to_string_pretty};
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use crate::connection_manager::TurtleCommBus;
pub type WsSend = SplitSink<WebSocketStream<TcpStream>, Message>;
pub type WsRecv = SplitStream<WebSocketStream<TcpStream>>;

pub struct ServerTurtle {
    inner: Turtle,
    #[allow(dead_code)]
    send: WsSend,
    comm_bus: UnboundedSender<TurtleCommBus>,
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
        let mut turtle = ServerTurtle {
            inner,
            send,
            comm_bus,
        };
        turtle.init(recv).await;
        turtle
    }

    pub async fn on_msg_recived(&mut self, msg: T2SPackets) {
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
                    MoveDirection::Down => self.inner.position += Pos3::new(0, -1, 0),
                    MoveDirection::Left => self.inner.orientation = self.inner.turn(TurnDir::Left),
                    MoveDirection::Right => {
                        self.inner.orientation = self.inner.turn(TurnDir::Right)
                    }
                };
                info!("moved: {:#?}", self.inner);
                _ = self.comm_bus.send(TurtleCommBus::Moved(self.index)).await;
                _ = self
                    .comm_bus
                    .send(TurtleCommBus::UpdateBlock(Block::new(None, &self.position)))
                    .await;
            }
            T2SPackets::Blocks { up, down, front } => {
                info!("up: {:?}", up);
                info!("front: {:?}", front);
                info!("down: {:?}", down);
                use TurtleCommBus::UpdateBlock;
                _ = self
                    .comm_bus
                    .send(UpdateBlock(Block::new(
                        up.into(),
                        &(self.position + Pos3::new(0, 1, 0)),
                    )))
                    .await;
                _ = self
                    .comm_bus
                    .send(UpdateBlock(Block::new(
                        front.into(),
                        &(self.position + self.get_forward_vec()),
                    )))
                    .await;
                _ = self
                    .comm_bus
                    .send(UpdateBlock(Block::new(
                        down.into(),
                        &(self.position + Pos3::new(0, -1, 0)),
                    )))
                    .await;
            }
        }
    }
    #[allow(dead_code)]
    async fn send_ws(&mut self, packet: S2TPackets) {
        self.send
            .send(Message::Text(to_string_pretty(&packet).unwrap()))
            .await
            .unwrap();
    }

    pub async fn move_(&mut self, dir: MoveDirection) {
        self.send_ws(S2TPackets::Move(vec![dir])).await;
    }

    async fn init(&mut self, mut recv: WsRecv) {
        let mut channel = self.comm_bus.clone();
        let index = self.index.clone();
        tokio::spawn(async move {
            loop {
                let packet = recv.next().await;
                match packet {
                    Some(Ok(Message::Text(msg))) => {
                        if let Ok(msg) = from_str::<T2SPackets>(&msg) {
                            channel
                                .send(TurtleCommBus::Packet((index, msg)))
                                .await
                                .unwrap();
                        }
                    }
                    None => {
                        _ = channel.send(TurtleCommBus::RemoveMe(index)).await;
                    }
                    Some(Err(_)) => {
                        _ = channel.send(TurtleCommBus::RemoveMe(index)).await;
                    }
                    Some(_) => {}
                }
            }
        });
    }
}
