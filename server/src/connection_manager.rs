use std::{path::Path, sync::Arc};

use crate::data_types::arc_mutex::ArcMutex;
use crate::data_types::client_map;
use crate::data_types::server_client::ServerClient;
use crate::data_types::server_turtle::{ServerTurtle, WsRecv, WsSend};
use crate::db::DB;
use common::turtle::Turtle;
use common::turtle_packets::{InfoData, T2SPackets};
use common::Pos3;
use futures_channel::mpsc::unbounded;
use futures_util::stream::{SplitSink, SplitStream};
use log::info;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

pub enum TurtleCommBus {
    Moved(i32),
}

pub async fn main(
    mut new_turte_connected: UnboundedReceiver<(InfoData, WsSend, WsRecv)>,
) -> anyhow::Result<()> {
    let db = Arc::new(Mutex::new(DB::new(Path::new("./db.json"))?));
    let server_turtles = Arc::new(Mutex::new(Vec::<ServerTurtle>::new()));
    let server_clients = Arc::new(Mutex::new(client_map::ClientMap::new()));
    let (turtle_comms_tx, turtle_comms_rx) = unbounded::<TurtleCommBus>();

    //Wait for Turtles to connect
    let local_db = db.clone();
    let local_server_turtles = server_turtles.clone();
    tokio::spawn(async move {
        while let Some((info_data, send, recv)) = new_turte_connected.recv().await {
            accept_turtle(
                info_data,
                local_db.clone(),
                local_server_turtles.clone(),
                send,
                recv,
                turtle_comms_tx.clone(),
            )
            .await;
        }
    });

    anyhow::Ok(())
}

async fn accept_turtle(
    info_data: InfoData,
    db: Arc<Mutex<DB>>,
    server_turtles: Arc<Mutex<Vec<ServerTurtle>>>,
    send: SplitSink<WebSocketStream<TcpStream>, Message>,
    recv: SplitStream<WebSocketStream<TcpStream>>,
    comm_bus: futures_channel::mpsc::UnboundedSender<TurtleCommBus>,
) {
    let mut db = db.lock().await;
    let mut server_turtles = server_turtles.lock().await;
    info!("new turtle with index: {}", info_data.index);
    if db.contains_turtle(info_data.index) {
        info!("turtle exists");
        let t = db.get_turtle(info_data.index).unwrap();
        server_turtles.push(ServerTurtle::new(t.clone_arc(), send, recv, comm_bus));
        ServerTurtle::on_msg_recived(t.clone_arc(), T2SPackets::Info(info_data));
    } else {
        info!("turtle dosen't exist ... yet");

        let InfoData {
            index,
            name,
            inventory,
            fuel,
            max_fuel,
        } = info_data;

        let inner = Turtle::new(
            index,
            name,
            inventory,
            Pos3::ZERO,
            common::turtle::Orientation::North,
            fuel,
            max_fuel,
        );
        let inner = ArcMutex::new(inner);
        db.push_turtle(inner.clone_arc()).unwrap();
        server_turtles.push(ServerTurtle::new(inner.clone_arc(), send, recv, comm_bus));
    };
}
