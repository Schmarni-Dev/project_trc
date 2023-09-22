use std::sync::Arc;

use crate::data_types::client_map;

use crate::data_types::server_client::{ClientComms, ServerClient};
use crate::data_types::server_turtle::{ServerTurtle, WsRecv, WsSend};
use crate::data_types::turtle_map::TurtleMap;
use crate::db::{DBError, DB};

use common::client_packets::{C2SPackets, MovedTurtleData, S2CPackets, SetTurtlesData};
use common::turtle::Turtle;
use common::turtle_packets::{SetupInfoData, T2SPackets};
use common::world_data::Block;

use futures_channel::mpsc::unbounded;

use futures_util::{pin_mut, SinkExt, StreamExt};
use log::{error, info};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;

pub enum TurtleCommBus {
    /// stupid fucking workaround. cant do this in ServerTurtle because the borrow checker; That fuck
    Packet((i32, T2SPackets)),
    Moved(i32),
    RemoveMe(i32),
    InvUpdate(i32),
    FuelUpdate(i32),
    UpdateBlock(Block),
}

pub async fn main(
    mut new_turte_connected: UnboundedReceiver<(SetupInfoData, Vec<T2SPackets>, WsSend, WsRecv)>,
    mut new_client_connected: UnboundedReceiver<(WsSend, WsRecv)>,
    db: Arc<DB>,
) -> anyhow::Result<()> {
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
            match comms {
                ClientComms::KILL_ME => {
                    local_server_clients
                        .lock()
                        .await
                        .execute_the_client(&client_index)
                        .await;
                }
                ClientComms::Packet(packet) => match packet {
                    C2SPackets::MoveTurtle {
                        index,
                        world,
                        direction,
                    } => {
                        if let Some(t) = local_server_turtles
                            .lock()
                            .await
                            .get_turtle_mut_id_and_world(index, &world)
                        {
                            t.move_(direction).await;
                        }
                    }
                    C2SPackets::RequestTurtles(world) => {
                        let mut online_turtles = local_server_turtles
                            .lock()
                            .await
                            .get_common_turtles()
                            .into_iter()
                            .map(|mut t| {
                                t.is_online = true;
                                t
                            })
                            .filter(|t| t.world == world)
                            .collect::<Vec<_>>();
                        let indexes = online_turtles.iter().map(|t| t.index).collect::<Vec<_>>();
                        let mut turtles = match local_db.get_turtles(&world).await {
                            Ok(t) => t
                                .into_iter()
                                .filter(|t| !indexes.contains(&t.index))
                                .collect(),
                            Err(err) => {
                                error!("{err}");
                                Vec::new()
                            }
                        };
                        turtles.append(&mut online_turtles);
                        info!("{turtles:#?}");
                        local_server_clients
                            .lock()
                            .await
                            .broadcast(S2CPackets::SetTurtles(SetTurtlesData { turtles, world }))
                            .await;
                    }
                    C2SPackets::RequestWorld(name) => {
                        local_server_clients
                            .lock()
                            .await
                            .send_to(
                                S2CPackets::SetWorld(local_db.get_world(&name).await.unwrap()),
                                &client_index,
                            )
                            .await;
                    }
                    C2SPackets::RequestWorlds => {
                        let worlds = match local_db.clone().get_worlds().await {
                            Ok(o) => o,
                            Err(e) => {
                                error!("{e}");
                                Vec::new()
                            }
                        };
                        local_server_clients
                            .lock()
                            .await
                            .send_to(S2CPackets::Worlds(worlds), &client_index)
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
                TurtleCommBus::RemoveMe(index) => {
                    info!("Killing(?) trutle: {} ", &index);
                    local_server_turtles.lock().await.drop_turtle(&index);
                }

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
                        index: t.index,
                        new_orientation: t.orientation,
                        new_pos: t.position,
                        world: t.world.clone(),
                    };
                    local_server_clients
                        .lock()
                        .await
                        .broadcast(S2CPackets::MovedTurtle(msg))
                        .await;
                }
                TurtleCommBus::UpdateBlock(block) => {
                    local_db.set_block(&block).await;
                    local_server_clients
                        .lock()
                        .await
                        .broadcast(S2CPackets::WorldUpdate(block))
                        .await;
                }
                TurtleCommBus::InvUpdate(index) => {
                    let sts = local_server_turtles.lock().await;
                    let t = sts.get_turtle(index);
                    if let Some(t) = t {
                        local_server_clients
                            .lock()
                            .await
                            .broadcast(S2CPackets::TurtleInventoryUpdate(
                                common::client_packets::UpdateTurtleData {
                                    index: t.index,
                                    world: t.world.clone(),
                                    data: t.inventory.clone(),
                                },
                            ))
                            .await;
                    }
                }
                TurtleCommBus::FuelUpdate(index) => {
                    let sts = local_server_turtles.lock().await;
                    let t = sts.get_turtle(index);
                    if let Some(t) = t {
                        local_server_clients
                            .lock()
                            .await
                            .broadcast(S2CPackets::TurtleFuelUpdate(
                                common::client_packets::UpdateTurtleData {
                                    index: t.index,
                                    world: t.world.clone(),
                                    data: t.fuel,
                                },
                            ))
                            .await;
                    }
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
        while let Some((info, data, mut send, recv)) = new_turte_connected.recv().await {
            let db = local_db.clone();
            let mut server_turtles = local_server_turtles.lock().await;
            info!("new turtle with index: {}", info.index);

            let t = match db.get_turtle(info.index, &info.world).await {
                Ok(turtle) => turtle,
                Err(DBError::NotFound) => db
                    .get_dummy_turtle(info.index, info.world, info.position, info.facing)
                    .await
                    .unwrap(),
                Err(DBError::Error(err)) => {
                    error!("{}", err);
                    send.close().await;
                    continue;
                }
            };

            let mut st =
                ServerTurtle::new(t, send, recv, turtle_comms_tx.clone(), db.clone()).await;
            st.on_msg_recived(T2SPackets::Batch(data)).await.unwrap();
            let t: Turtle = st.to_owned();
            server_turtles.push(st);
            info!("???");
            let world = t.world;
            let mut online_turtles = server_turtles
                .get_common_turtles()
                .into_iter()
                .map(|mut t| {
                    t.is_online = true;
                    t
                })
                .filter(|t| t.world == world)
                .collect::<Vec<_>>();
            let indexes = online_turtles.iter().map(|t| t.index).collect::<Vec<_>>();
            let mut turtles = match local_db.get_turtles(&world).await {
                Ok(t) => t
                    .into_iter()
                    .filter(|t| !indexes.contains(&t.index))
                    .collect(),
                Err(err) => {
                    error!("{err}");
                    Vec::new()
                }
            };
            turtles.append(&mut online_turtles);
            info!("connect set turtles{turtles:#?}");
            local_server_clients
                .lock()
                .await
                .broadcast(S2CPackets::SetTurtles(SetTurtlesData { turtles, world }))
                .await;
        }
    });
    //
    // let mut db_save_to_disk = tokio::time::interval(Duration::minutes(5).to_std().unwrap());
    // db_save_to_disk.tick().await;
    // let local_server_turtles = server_turtles.clone();
    // let local_db = db.clone();
    // tokio::spawn(async move {
    //     loop {
    //         db_save_to_disk.tick().await;
    //         save_db(local_db.clone(), local_server_turtles.clone()).await;
    //     }
    // });

    // let local_server_turtles = server_turtles.clone();
    // let local_db = db.clone();
    // std::thread::spawn(move || loop {
    //     let mut input = String::new();
    //     match std::io::stdin().read_line(&mut input) {
    //         Ok(_) => match trim_newline(input).as_str() {
    //             "quit" | "q" | "exit" => {
    //                 let local_server_turtles = local_server_turtles.clone();
    //                 let local_db = local_db.clone();
    //                 let rt = tokio::runtime::Runtime::new().unwrap();
    //                 rt.block_on(async move {
    //                     save_db(local_db, local_server_turtles).await;
    //                 });
    //                 std::process::exit(0);
    //             }
    //             w => {
    //                 println!("{:?}", w);
    //             }
    //         },
    //         Err(error) => println!("error: {error}"),
    //     }
    // });

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

// async fn save_db(db: Arc<Mutex<DB>>, turtles: Arc<Mutex<TurtleMap>>) {
//     let mut db = db.lock().await;
//     turtles
//         .lock()
//         .await
//         .get_common_turtles()
//         .into_iter()
//         .for_each(|t| db.push_turtle(t).unwrap());
//     db.save().expect("!!!DB FALIED TO SAVE!!!");
// }

// async fn accept_turtle(
//     info_data: InfoData,
//     db_mutex: Arc<Mutex<DB>>,
//     server_turtles_mutex: Arc<Mutex<TurtleMap>>,
//     send: SplitSink<WebSocketStream<TcpStream>, Message>,
//     recv: SplitStream<WebSocketStream<TcpStream>>,
//     comm_bus: futures_channel::mpsc::UnboundedSender<TurtleCommBus>,
// ) {
//     let mut db = db_mutex.lock().await;
//     let mut server_turtles = server_turtles_mutex.lock().await;
//     info!("new turtle with index: {}", info_data.index);
//     if db.contains_turtle(info_data.index) {
//         info!("turtle exists");
//         let t = db.get_turtle(info_data.index).unwrap();
//         drop(db);
//         let mut st = ServerTurtle::new(t, send, recv, comm_bus).await;
//         st.on_msg_recived(T2SPackets::Info(info_data)).await;
//         server_turtles.push(st);
//     } else {
//         info!("turtle dosen't exist ... yet");
//
//         let InfoData {
//             index,
//             name,
//             inventory,
//             fuel,
//             max_fuel,
//         } = &info_data;
//
//         let inner = Turtle::new(
//             index.to_owned(),
//             name.to_owned(),
//             inventory.to_owned(),
//             Pos3::ZERO,
//             Orientation::North,
//             fuel.to_owned(),
//             max_fuel.to_owned(),
//             true,
//         );
//         db.push_turtle(inner.clone()).unwrap();
//         drop(db);
//         let mut st = ServerTurtle::new(inner, send, recv, comm_bus).await;
//         st.on_msg_recived(T2SPackets::Info(info_data)).await;
//         server_turtles.push(st);
//     };
// }
