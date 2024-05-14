use std::sync::Arc;

use crate::data_types::client_map;

use crate::data_types::server_client::{ClientComms, ServerClient};
use crate::data_types::server_turtle::{ServerTurtle, WsRecv, WsSend};
use crate::data_types::turtle_map::TurtleMap;
use crate::db::{pos_to_db_pos, pos_to_key, DB};

use common::client_packets::{C2SPackets, MovedTurtleData, S2CPackets, SetTurtlesData};
use common::turtle::{Maybe, Turtle};
use common::turtle_packets::{S2TPackets, SetupInfoData, T2SPackets};
use common::world_data::{get_chunk_containing_block, Block, World};

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
                    info!("/kill @e[type=client,id={}]", client_index);
                    local_server_clients
                        .lock()
                        .await
                        .execute_the_client(&client_index)
                        .await;
                }
                ClientComms::Packet(packet) => match packet {
                    // C2SPackets::BreakBlock { index, world, dir } => {
                    //     if let Some(t) = local_server_turtles
                    //         .lock()
                    //         .await
                    //         .get_turtle_mut_id_and_world(index, &world)
                    //     {
                    //         t.send_ws(S2TPackets::BreakBlock { dir }).await;
                    //     }
                    // }
                    // C2SPackets::PlaceBlock {
                    //     index,
                    //     world,
                    //     dir,
                    //     text,
                    // } => {
                    //     if let Some(t) = local_server_turtles
                    //         .lock()
                    //         .await
                    //         .get_turtle_mut_id_and_world(index, &world)
                    //     {
                    //         t.send_ws(S2TPackets::PlaceBlock { dir, text }).await;
                    //     }
                    // }
                    // C2SPackets::MoveTurtle {
                    //     index,
                    //     world,
                    //     direction,
                    // } => {
                    //     if let Some(t) = local_server_turtles
                    //         .lock()
                    //         .await
                    //         .get_turtle_mut_id_and_world(index, &world)
                    //     {
                    //         t.move_(direction).await;
                    //     }
                    // }
                    C2SPackets::RequestTurtles(world) => {
                        let indexes = local_server_turtles
                            .lock()
                            .await
                            .get_common_turtles()
                            .into_iter()
                            .filter(|t| t.world == world)
                            .map(|t| t.index)
                            .collect::<Vec<_>>();
                        use crate::db::DbTurtle;
                        let w = sqlx::query_as!(
                            DbTurtle,
                            "SELECT * FROM turtles WHERE world = ?",
                            world
                        )
                        .fetch_all(&*local_db)
                        .await;
                        let turtles = match w {
                            Ok(t) => t
                                .into_iter()
                                .map(Turtle::from)
                                .map(|mut t| {
                                    t.is_online = indexes.contains(&t.index);
                                    t
                                })
                                .collect::<Vec<_>>(),
                            Err(err) => {
                                error!("{err}");
                                Vec::new()
                            }
                        };
                        local_server_clients
                            .lock()
                            .await
                            .broadcast(S2CPackets::SetTurtles(SetTurtlesData { turtles, world }))
                            .await;
                    }
                    C2SPackets::RequestWorld(name) => {
                        use crate::db::DbBlock;
                        let mut world = World::new(&name);
                        let block_iter =
                            sqlx::query_as!(DbBlock, "SELECT * FROM blocks WHERE world = ?", name)
                                .fetch_all(&*local_db)
                                .await
                                .unwrap()
                                .into_iter()
                                .map(Block::from);
                        for block in block_iter {
                            world.set_block(block);
                        }
                        local_server_clients
                            .lock()
                            .await
                            .send_to(S2CPackets::SetWorld(world), &client_index)
                            .await;
                    }
                    C2SPackets::RequestWorlds => {
                        let worlds = match sqlx::query!("SELECT name FROM worlds;")
                            .fetch_all(&*local_db)
                            .await
                            .map(|r| r.into_iter().map(|r| r.name).collect::<Vec<_>>())
                        {
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
                    // C2SPackets::TurtleSelectSlot { index, world, slot } => {
                    //     if let Some(t) = local_server_turtles
                    //         .lock()
                    //         .await
                    //         .get_turtle_mut_id_and_world(index, &world)
                    //     {
                    //         t.send_ws(S2TPackets::SelectSlot(slot)).await;
                    //     }
                    // }
                    C2SPackets::SendLuaToTurtle { index, world, code } => {
                        if let Some(t) = local_server_turtles
                            .lock()
                            .await
                            .get_turtle_mut_id_and_world(index, &world)
                        {
                            t.send_ws(S2TPackets::RunLuaCode(code)).await;
                        }
                    }
                    C2SPackets::StdInForTurtle { index, value } => todo!(),
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
                    info!("/kill @e[type=trutle,id={}] ", &index);
                    let mut server_turtles = local_server_turtles.lock().await;
                    let world = server_turtles.drop_turtle(&index).map(|t| t.world.clone());
                    if let Some(world) = world {
                        let indexes = local_server_turtles
                            .lock()
                            .await
                            .get_common_turtles()
                            .into_iter()
                            .filter(|t| t.world == world)
                            .map(|t| t.index)
                            .collect::<Vec<_>>();
                        use crate::db::DbTurtle;
                        let w = sqlx::query_as!(
                            DbTurtle,
                            "SELECT * FROM turtles WHERE world = ?",
                            world
                        )
                        .fetch_all(&*local_db)
                        .await;
                        let turtles = match w {
                            Ok(t) => t
                                .into_iter()
                                .map(Turtle::from)
                                .map(|mut t| {
                                    t.is_online = indexes.contains(&t.index);
                                    t
                                })
                                .collect::<Vec<_>>(),
                            Err(err) => {
                                error!("{err}");
                                Vec::new()
                            }
                        };
                        local_server_clients
                            .lock()
                            .await
                            .broadcast(S2CPackets::SetTurtles(SetTurtlesData { turtles, world }))
                            .await;
                    }
                }

                TurtleCommBus::Packet((i, p)) => {
                    match local_server_turtles
                        .lock()
                        .await
                        .get_turtle_mut(i)
                        .unwrap()
                        .on_msg_recived(p)
                        .await
                    {
                        Ok(_) => (),
                        Err(e) => error!("Trutle Packet Err: {e}"),
                    }
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
                    // let _ = local_db.set_block(&block).await;
                    let chunk_key = pos_to_key(&get_chunk_containing_block(&block.pos));
                    let db_pos = pos_to_db_pos(&block.pos);
                    let _ = sqlx::query!(
                        "INSERT OR REPLACE INTO blocks VALUES (?,?,?,?,?);",
                        chunk_key,
                        block.id,
                        block.world,
                        db_pos,
                        block.is_air,
                    )
                    .execute(&*local_db)
                    .await;
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
                        if let Maybe::Some(inv) = t.inventory.clone() {
                            local_server_clients
                                .lock()
                                .await
                                .broadcast(S2CPackets::TurtleInventoryUpdate(
                                    common::client_packets::UpdateTurtleData {
                                        index: t.index,
                                        world: t.world.clone(),
                                        data: inv,
                                    },
                                ))
                                .await;
                        }
                    }
                }
                TurtleCommBus::FuelUpdate(index) => {
                    info!("fuel {index}");
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
            use crate::db::DbTurtle;
            let db_turtle = sqlx::query_as!(
                DbTurtle,
                "SELECT * FROM turtles WHERE id = ? AND world = ?;",
                info.index,
                info.world
            )
            .fetch_one(&*local_db)
            .await;
            let t = match db_turtle.map(Turtle::from) {
                Ok(turtle) => turtle,
                Err(sqlx::Error::RowNotFound) => {
                    let dummy =
                        Turtle::new_dummy(info.index, info.world, info.position, info.facing);
                    let db_pos = pos_to_db_pos(&dummy.position);
                    let orient_str = dummy.orientation.to_string();
                    // TODO: Check if this fails and do something
                    let _ =sqlx::query!(
                        "INSERT INTO turtles VALUES (?,?,?,?,?,?,?)",
                        dummy.index,
                        dummy.name,
                        db_pos,
                        orient_str,
                        dummy.fuel,
                        dummy.max_fuel,
                        dummy.world
                    )
                    .execute(&*local_db)
                    .await;
                    dummy
                }
                Err(err) => {
                    error!("{}", err);
                    send.close().await.unwrap();
                    continue;
                }
            };

            let mut st =
                ServerTurtle::new(t, send, recv, turtle_comms_tx.clone(), db.clone()).await;
            st.on_msg_recived(T2SPackets::Batch(data)).await.unwrap();
            let t: Turtle = st.to_owned();
            server_turtles.push(st);
            let world = t.world;
            let indexes = local_server_turtles
                .lock()
                .await
                .get_common_turtles()
                .into_iter()
                .filter(|t| t.world == world)
                .map(|t| t.index)
                .collect::<Vec<_>>();
            let w = sqlx::query_as!(DbTurtle, "SELECT * FROM turtles WHERE world = ?", world)
                .fetch_all(&*local_db)
                .await;
            let turtles = match w {
                Ok(t) => t
                    .into_iter()
                    .map(Turtle::from)
                    .map(|mut t| {
                        t.is_online = indexes.contains(&t.index);
                        t
                    })
                    .collect::<Vec<_>>(),
                Err(err) => {
                    error!("{err}");
                    Vec::new()
                }
            };
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

// fn trim_newline(mut s: String) -> String {
//     if s.ends_with('\n') {
//         s.pop();
//         if s.ends_with('\r') {
//             s.pop();
//         }
//     };
//     s
// }

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
