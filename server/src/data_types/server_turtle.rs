use std::{
    ops::Deref,
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
    inner: ArcMutex<Turtle>,
    send: Arc<Mutex<WsSend>>,
    // recv:WsRecv,
}

impl ServerTurtle {
    /// Check if inner exists allready in the db, if not make new Turtle else load Turtle from db!
    pub async fn new(
        inner: ArcMutex<Turtle>,
        send: WsSend,
        recv: WsRecv,
        comm_bus: UnboundedSender<TurtleCommBus>,
    ) -> ServerTurtle {
        let turtle = ServerTurtle {
            inner,
            send: Arc::new(Mutex::new(send)),
        };
        turtle.init(recv);
        turtle
    }

    pub(crate) async fn on_msg_recived(inner: ArcMutex<Turtle>, msg: T2SPackets) {
        match msg {
            T2SPackets::Info(InfoData {
                index: _,
                name,
                inventory,
                fuel,
                max_fuel,
            }) => {
                let mut inner = inner.0.lock().unwrap();
                inner.fuel = fuel;
                inner.max_fuel = max_fuel;
                inner.inventory = inventory;
                inner.name = name;
                info!("Info Recived ^^7")
            }
            T2SPackets::Moved { direction } => {
                //TODO: Somehow Notify client of Change
                let mut inner = inner.0.lock().unwrap();
                match direction {
                    MoveDirection::Forward => {
                        let forward = inner.get_forward_vec();
                        inner.position += forward;
                    }
                    MoveDirection::Back => {
                        let forward = inner.get_forward_vec();
                        inner.position -= forward;
                    }
                    MoveDirection::Up => inner.position += Pos3::new(0, 1, 0),
                    MoveDirection::Down => inner.position -= Pos3::new(0, 1, 0),
                    MoveDirection::Left => inner.orientation = inner.turn(TurnDir::Left),
                    MoveDirection::Right => inner.orientation = inner.turn(TurnDir::Right),
                }
            }
            T2SPackets::Blocks { up, down, front } => {
                info!("up: {:?}", up);
                info!("front: {:?}", front);
                info!("down: {:?}", down);
            }
        }
    }

    async fn init(&self, recv: WsRecv) {
        // V Actually yes Just the logging was FUCKED V
        // info!("am i ever inniting? .. no");

        let _send = self.send.clone();
        let inner = self.inner.clone_arc();
        tokio::spawn(async move {
            recv.try_for_each(|msg| {
                if let Message::Text(msg) = msg {
                    info!("recived msg: {}", msg);
                    ServerTurtle::on_msg_recived(
                        inner.clone_arc(),
                        from_str::<T2SPackets>(&msg).unwrap(),
                    );
                }
                future::ok(())
            })
            .await
        });
    }
    pub fn get_pos(&self) -> Pos3 {
        self.inner.0.lock().unwrap().position
    }
}
