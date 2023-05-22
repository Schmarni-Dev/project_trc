use std::{path::Path, sync::Arc};

use crate::data_types::client_map;

use crate::data_types::server_client::{ClientComms, ServerClient};
use crate::data_types::server_turtle::{ServerTurtle, WsRecv, WsSend};
use crate::data_types::turtle_map::TurtleMap;
use crate::db::DB;
use chrono::Duration;
use common::client_packets::{C2SPackets, MovedTurtleData, S2CPackets};
use common::turtle::Turtle;
use common::turtle_packets::{InfoData, T2SPackets};
use common::Pos3;
use futures_channel::mpsc::unbounded;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{pin_mut, StreamExt};
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
    mut new_client_connected: UnboundedReceiver<(WsSend, WsRecv)>,
) -> anyhow::Result<()> {
    let db = Arc::new(Mutex::new(DB::new(Path::new("./db.json"))?));
    let server_turtles = Arc::new(Mutex::new(TurtleMap::new()));
    let server_clients = Arc::new(Mutex::new(client_map::ClientMap::new()));
    let (turtle_comms_tx, mut turtle_comms_rx) = unbounded::<TurtleCommBus>();
    let (client_comms_tx, mut client_comms_rx) = unbounded::<(i32, ClientComms)>();
    pin_mut!(turtle_comms_tx, client_comms_tx);

    let local_db = db.clone();
    let local_server_turtles = server_turtles.clone();
    let local_server_clients = server_clients.clone();
    tokio::spawn(async move {
        while let Some(w) = client_comms_rx.next().await {
            let (client_index, comms) = w;
            info!("?!: {:#?}", comms);
            match comms {
                ClientComms::KILL_ME => {}
                ClientComms::Packet(packet) => match packet {
                    C2SPackets::MoveTurtle { index, direction } => {
                        info!("No Lock?... {}", index);
                        if let Some(t) = local_server_turtles.lock().await.get_turtle_mut(index) {
                            info!("This Bitch dir: {:#?}", direction);
                            t.move_(direction).await;
                        }
                        info!("Yes Lock?");
                    }
                    C2SPackets::RequestTurtles => {
                        info!("This Bitch: {}", client_index);
                        local_server_clients
                            .lock()
                            .await
                            .send_to(
                                S2CPackets::RequestedTurtles(
                                    local_db.lock().await.get_turtles().unwrap(),
                                ),
                                &client_index,
                            )
                            .await;
                    }
                },
            }
        }
    });

    let local_db = db.clone();
    let local_server_clients = server_clients.clone();
    tokio::spawn(async move {
        while let Some(w) = turtle_comms_rx.next().await {
            match w {
                TurtleCommBus::Moved(index) => {
                    let t = local_db.lock().await.get_turtle(index).unwrap();
                    let msg = MovedTurtleData {
                        index,
                        new_orientation: t.orientation,
                        new_pos: t.position,
                    };
                    local_server_clients
                        .lock()
                        .await
                        .broadcast(S2CPackets::MovedTurtle(msg))
                        .await;
                }
            }
        }
    });

    let client_comms_tx = client_comms_tx.clone();
    tokio::spawn(async move {
        while let Some((send, recv)) = new_client_connected.recv().await {
            server_clients.lock().await.push(ServerClient::new(
                recv,
                send,
                client_comms_tx.clone(),
            ));
        }
    });

    //Wait for Turtles to connect
    let turtle_comms_tx = turtle_comms_tx.clone();
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

    let mut db_save_to_disk = tokio::time::interval(Duration::minutes(5).to_std().unwrap());
    db_save_to_disk.tick().await;
    let local_db = db.clone();
    tokio::spawn(async move {
        loop {
            db_save_to_disk.tick().await;
            local_db
                .lock()
                .await
                .save()
                .expect("!!!DB FALIED TO SAVE!!!");
        }
    });

    anyhow::Ok(())
}

async fn accept_turtle(
    info_data: InfoData,
    db_mutex: Arc<Mutex<DB>>,
    server_turtles_mutex: Arc<Mutex<TurtleMap>>,
    send: SplitSink<WebSocketStream<TcpStream>, Message>,
    recv: SplitStream<WebSocketStream<TcpStream>>,
    comm_bus: futures_channel::mpsc::UnboundedSender<TurtleCommBus>,
) {
    let mut db = db_mutex.lock().await;
    let mut server_turtles = server_turtles_mutex.lock().await;
    info!("new turtle with index: {}", info_data.index);
    if db.contains_turtle(info_data.index) {
        info!("turtle exists");
        let t = db.get_turtle(info_data.index).unwrap();
        drop(db);
        info!("Build!");
        let mut st = ServerTurtle::new(t, send, recv, comm_bus).await;
        info!("Send!");
        st.on_msg_recived(T2SPackets::Info(info_data)).await;
        info!("Push!");
        server_turtles.push(st);
        info!("Pushed!");
    } else {
        info!("turtle dosen't exist ... yet");

        let InfoData {
            index,
            name,
            inventory,
            fuel,
            max_fuel,
        } = &info_data;

        let inner = Turtle::new(
            index.to_owned(),
            name.to_owned(),
            inventory.to_owned(),
            Pos3::ZERO,
            common::turtle::Orientation::North,
            fuel.to_owned(),
            max_fuel.to_owned(),
        );
        db.push_turtle(inner.clone()).unwrap();
        drop(db);
        let mut st = ServerTurtle::new(inner, send, recv, comm_bus).await;
        st.on_msg_recived(T2SPackets::Info(info_data)).await;
        server_turtles.push(st);
    };
}
