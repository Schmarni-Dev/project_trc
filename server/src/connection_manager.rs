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
use common::world_data::Block;
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
    /// stupid fucking workaround. cant do this in ServerTurtle because the borrow checker; That mo'fucker
    Packet((i32, T2SPackets)),
    Moved(i32),
    RemoveMe,
    UpdateBlock((Pos3, Block)),
}

pub async fn main(
    mut new_turte_connected: UnboundedReceiver<(InfoData, WsSend, WsRecv)>,
    mut new_client_connected: UnboundedReceiver<(WsSend, WsRecv)>,
) -> anyhow::Result<()> {
    // Db is For Presistent storage only!
    let db = Arc::new(Mutex::new(DB::new(Path::new("./db.json"))?));
    let server_turtles = Arc::new(Mutex::new(TurtleMap::new()));
    let server_clients = Arc::new(Mutex::new(client_map::ClientMap::new()));
    let (turtle_comms_tx, mut turtle_comms_rx) = unbounded::<TurtleCommBus>();
    let (client_comms_tx, mut client_comms_rx) = unbounded::<(i32, ClientComms)>();
    pin_mut!(turtle_comms_tx, client_comms_tx);

    // let local_db = db.clone();
    let local_server_turtles = server_turtles.clone();
    let local_server_clients = server_clients.clone();
    tokio::spawn(async move {
        while let Some(w) = client_comms_rx.next().await {
            let (client_index, comms) = w;
            match comms {
                ClientComms::KILL_ME => {}
                ClientComms::Packet(packet) => match packet {
                    C2SPackets::MoveTurtle { index, direction } => {
                        if let Some(t) = local_server_turtles.lock().await.get_turtle_mut(index) {
                            t.move_(direction).await;
                        }
                    }
                    C2SPackets::RequestTurtles => {
                        local_server_clients
                            .lock()
                            .await
                            .send_to(
                                S2CPackets::RequestedTurtles(
                                    local_server_turtles.lock().await.get_common_turtles(),
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
    let local_server_turtles = server_turtles.clone();
    tokio::spawn(async move {
        while let Some(w) = turtle_comms_rx.next().await {
            match w {
                TurtleCommBus::RemoveMe => {}

                TurtleCommBus::Packet((i, p)) => {
                    local_server_turtles
                        .lock()
                        .await
                        .get_turtle_mut(i)
                        .unwrap()
                        .on_msg_recived(p)
                        .await;
                }
                TurtleCommBus::Moved(index) => {
                    let sts = local_server_turtles.lock().await;
                    let t = sts.get_turtle(index).unwrap();
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
                TurtleCommBus::UpdateBlock((pos, block)) => {
                    local_db.lock().await;
                }
            }
        }
    });

    let client_comms_tx = client_comms_tx.clone();
    let local_server_clients = server_clients.clone();
    tokio::spawn(async move {
        while let Some((send, recv)) = new_client_connected.recv().await {
            local_server_clients.lock().await.push(ServerClient::new(
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
    let local_server_clients = server_clients.clone();
    tokio::spawn(async move {
        while let Some((info_data, send, recv)) = new_turte_connected.recv().await {
            let mut db = local_db.lock().await;
            let mut server_turtles = local_server_turtles.lock().await;
            info!("new turtle with index: {}", info_data.index);
            if db.contains_turtle(info_data.index) {
                info!("turtle exists");
                let t = db.get_turtle(info_data.index).unwrap();
                drop(db);
                local_server_clients
                    .lock()
                    .await
                    .broadcast(S2CPackets::TurtleConnected(t.clone()))
                    .await;
                let mut st = ServerTurtle::new(t, send, recv, turtle_comms_tx.clone()).await;
                st.on_msg_recived(T2SPackets::Info(info_data)).await;
                server_turtles.push(st);
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
                local_server_clients
                    .lock()
                    .await
                    .broadcast(S2CPackets::TurtleConnected(inner.clone()))
                    .await;
                let mut st = ServerTurtle::new(inner, send, recv, turtle_comms_tx.clone()).await;
                st.on_msg_recived(T2SPackets::Info(info_data)).await;
                server_turtles.push(st);
            };
            // accept_turtle(
            //     info_data,
            //     local_db.clone(),
            //     local_server_turtles.clone(),
            //     send,
            //     recv,
            //     turtle_comms_tx.clone(),
            // )
            // .await;
        }
    });

    let mut db_save_to_disk = tokio::time::interval(Duration::minutes(5).to_std().unwrap());
    db_save_to_disk.tick().await;
    let local_server_turtles = server_turtles.clone();
    let local_db = db.clone();
    tokio::spawn(async move {
        loop {
            db_save_to_disk.tick().await;
            save_db(local_db.clone(), local_server_turtles.clone()).await;
        }
    });

    let local_server_turtles = server_turtles.clone();
    let local_db = db.clone();
    std::thread::spawn(move || loop {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => match trim_newline(input).as_str() {
                "quit" | "q" | "exit" => {
                    let local_server_turtles = local_server_turtles.clone();
                    let local_db = local_db.clone();
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async move {
                        save_db(local_db, local_server_turtles).await;
                    });
                    std::process::exit(0);
                }
                w => {
                    println!("{:?}", w);
                }
            },
            Err(error) => println!("error: {error}"),
        }
    });

    anyhow::Ok(())
}

fn trim_newline(mut s: String) -> String {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    };
    s
}

async fn save_db(db: Arc<Mutex<DB>>, turtles: Arc<Mutex<TurtleMap>>) {
    let mut db = db.lock().await;
    turtles
        .lock()
        .await
        .get_common_turtles()
        .into_iter()
        .for_each(|t| db.push_turtle(t).unwrap());
    db.save().expect("!!!DB FALIED TO SAVE!!!");
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
        let mut st = ServerTurtle::new(t, send, recv, comm_bus).await;
        st.on_msg_recived(T2SPackets::Info(info_data)).await;
        server_turtles.push(st);
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
