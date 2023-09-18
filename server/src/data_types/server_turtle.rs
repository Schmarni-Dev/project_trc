use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use common::{
    turtle::{MoveDirection, TurnDir, Turtle},
    turtle_packets::{S2TPackets, SetupInfoData, T2SPackets},
    world_data::Block,
    Pos3,
};

use futures_channel::mpsc::UnboundedSender;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use libsql_client::{args, Client};
#[allow(unused_imports)]
use log::info;
use serde_json::{from_str, to_string_pretty};
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use crate::{
    connection_manager::TurtleCommBus,
    db::{pos_to_db_pos, DB},
};
pub type WsSend = SplitSink<WebSocketStream<TcpStream>, Message>;
pub type WsRecv = SplitStream<WebSocketStream<TcpStream>>;

pub struct ServerTurtle {
    db: Arc<DB>,
    inner: Turtle,
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
        db: Arc<DB>,
    ) -> ServerTurtle {
        let mut turtle = ServerTurtle {
            inner,
            send,
            comm_bus,
            db,
        };
        turtle.init(recv).await;
        turtle
    }

    #[inline(always)]
    async fn re_packet(&mut self, msg: T2SPackets) -> Result<(), futures_channel::mpsc::SendError> {
        self.comm(TurtleCommBus::Packet((self.index, msg))).await
    }
    #[inline(always)]
    async fn comm(&mut self, msg: TurtleCommBus) -> Result<(), futures_channel::mpsc::SendError> {
        self.comm_bus.send(msg).await
    }

    pub async fn on_msg_recived(&mut self, msg: T2SPackets) -> anyhow::Result<()> {
        match msg {
            T2SPackets::Batch(packets) => {
                for p in packets {
                    self.comm(TurtleCommBus::Packet((self.index, p)))
                        .await?;
                }
            }
            T2SPackets::SetPos(pos) => {
                self.position = pos;
                self.db
                    .exec(
                        "
                    UPDATE turtles SET position = ? 
                    WHERE index = ?, world = ?;
                    ",
                        args!(*pos_to_db_pos(&self.position), self.index, &self.world),
                    )
                    .await?;
            }
            T2SPackets::SetMaxFuel(max_fuel) => {
                self.max_fuel = max_fuel;
                self.db
                    .exec(
                        "
                    UPDATE turtles SET max_fuel = ? 
                    WHERE index = ?, world = ?;
                    ",
                        args!(max_fuel, self.index, &self.world),
                    )
                    .await?;
            }

            T2SPackets::SetOrientation(orient) => {
                self.orientation = orient;
                self.db
                    .exec(
                        "
                    UPDATE turtles SET orientation = ? 
                    WHERE index = ?, world = ?;
                    ",
                        args!(orient.to_string(), self.index, &self.world),
                    )
                    .await?;
            }
            T2SPackets::SetupInfo(SetupInfoData { .. }) => {}
            T2SPackets::InventoryUpdate(inv) => {
                self.inventory = inv;
                self.db
                    .exec(
                        "
                    UPDATE turtles SET inventory = ? 
                    WHERE index = ?, world = ?;
                    ",
                        args!(
                            serde_json::to_string(&self.inventory)?,
                            self.index,
                            &self.world
                        ),
                    )
                    .await?;
            }
            T2SPackets::WorldUpdate(w_name) => {
                self.db
                    .exec(
                        "
                    UPDATE turtles SET world = ? 
                    WHERE index = ?, world = ?;
                    ",
                        args!(&w_name, self.index, &self.world),
                    )
                    .await?;
                self.world = w_name;
            }
            T2SPackets::NameUpdate(name) => {
                self.name = name;
                self.db
                    .exec(
                        "
                    UPDATE turtles SET name = ? 
                    WHERE index = ?, world = ?;
                    ",
                        args!(&self.name, self.index, &self.world),
                    )
                    .await?;
            }
            T2SPackets::FuelUpdate(fuel) => {
                self.fuel = fuel;
                self.db
                    .exec(
                        "
                    UPDATE turtles SET fuel = ? 
                    WHERE index = ?, world = ?;
                    ",
                        args!(self.fuel, self.index, &self.world),
                    )
                    .await?;
            }
            T2SPackets::Moved { direction } => {
                let mut p = self.position.clone();
                let mut o = self.orientation.clone();
                match direction {
                    MoveDirection::Forward => {
                        let forward = self.inner.get_forward_vec();
                        p += forward;
                    }
                    MoveDirection::Back => {
                        let forward = self.inner.get_forward_vec();
                        p -= forward;
                    }
                    MoveDirection::Up => p += Pos3::new(0, 1, 0),
                    MoveDirection::Down => p += Pos3::new(0, -1, 0),
                    MoveDirection::Left => o = self.inner.turn(TurnDir::Left),
                    MoveDirection::Right => o = self.inner.turn(TurnDir::Right),
                };
                self.re_packet(T2SPackets::SetPos(p)).await?;
                self.re_packet(T2SPackets::SetOrientation(o)).await?;
                _ = self.comm_bus.send(TurtleCommBus::Moved(self.index)).await;
                self.comm(TurtleCommBus::UpdateBlock(Block::new(
                    None,
                    &self.position,
                    &self.world,
                )))
                .await?;
            }
            T2SPackets::Blocks { up, down, front } => {
                info!("up: {:?}", up);
                info!("front: {:?}", front);
                info!("down: {:?}", down);
                use TurtleCommBus::UpdateBlock;
                self.comm(UpdateBlock(Block::new(
                    up.into(),
                    &(self.position + Pos3::new(0, 1, 0)),
                    &self.world,
                )))
                .await?;
                self.comm(UpdateBlock(Block::new(
                    front.into(),
                    &(self.position + self.get_forward_vec()),
                    &self.world,
                )))
                .await?;
                self.comm(UpdateBlock(Block::new(
                    down.into(),
                    &(self.position + Pos3::new(0, -1, 0)),
                    &self.world,
                )))
                .await?;
            }
        }
        Ok(())
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
