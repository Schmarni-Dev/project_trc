use std::str::FromStr;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use bevy_ecs::{component::Component, entity::Entity};
use color_eyre::eyre::ContextCompat as _;
use common::{
    turtle::{get_rotated_orientation, Orientation, TurnDir},
    turtle_packets::S2TPacket,
    world_data::{chunk_pos_to_chunk_key, get_chunk_containing_block},
    ComputerType, Pos3,
};
use futures::{stream::SplitSink, SinkExt, StreamExt, TryStreamExt};
use log::{error, info, warn};
use sqlx::SqlitePool;
use trc_protocol::computer_packets::{ComputerSetupToServerPacket, TurtleToServerPacket};

use crate::{
    computers::{self, ComputerIdent},
    AppState,
};

pub async fn turtle_ws_route(ws: WebSocketUpgrade, state: State<AppState>) -> Response {
    ws.on_upgrade(|s| handle_turtle_ws(s, state))
}

pub async fn handle_turtle_ws(socket: WebSocket, state: State<AppState>) {
    // let session = state.sessions.acquire_session().await;
    let (writer, mut reader) = socket.split();
    let Some(Ok(Message::Text(msg))) = reader.next().await else {
        error!("invalid first message from turtle");
        return;
    };
    let Ok(setup): Result<ComputerSetupToServerPacket, _> = serde_json::from_str(&msg) else {
        error!("invalid setup packet: {msg}");
        return;
    };
    let entity = state
        .write()
        .await
        .spawn((
            TurtleWsSender(writer),
            ComputerType::Turtle,
            ComputerIdent {
                world: setup.world,
                id: setup.id,
            },
        ))
        .id();
    while let Ok(msg) = reader.try_next().await {
        let state = state.clone();
        let Some(msg) = msg else {
            warn!("turtle msg was None");
            continue;
        };
        match msg {
            Message::Text(text) => {
                let Ok(packet) = serde_json::from_str(&text) else {
                    error!("unable to parse turtle packet: {}", text);
                    continue;
                };
                if let Err(err) = handle_packet(packet, &state, entity).await {
                    error!("error while handling packet: {}", err);
                }
            }
            Message::Binary(_bin) => {}
            Message::Ping(_) => {}
            Message::Pong(_) => {}
            Message::Close(w) => {
                warn!("turtle ws closed because: {}", w.unwrap().reason);
                break;
            }
        }
    }
    info!("Turtle disconnected!");
    state.write().await.despawn(entity);
}

async fn handle_packet(
    packet: TurtleToServerPacket,
    state: &State<AppState>,
    entity: Entity,
) -> color_eyre::Result<()> {
    use TurtleToServerPacket as T2S;
    match packet {
        // T2SPackets::SetupInfo(SetupInfoData {
        //     facing,
        //     position,
        //     index,
        //     world,
        // }) => {
        //     let exits = sqlx::query!(
        //         "SELECT TRUE as stored FROM turtles WHERE id = ? AND world = ?;",
        //         index,
        //         world
        //     )
        //     .fetch_one(&state.db)
        //     .await
        //     .unwrap();
        //     let facing_str = facing.to_string();
        //     let pos_str = position.to_string_repr();
        //     if exits.stored != 0 {
        //         sqlx::query!(
        //             "UPDATE turtles SET orientation = ?, position = ? WHERE id = ? AND world = ?;",
        //             facing_str,
        //             pos_str,
        //             index,
        //             world
        //         )
        //         .execute(&state.db)
        //         .await
        //         .unwrap();
        //     } else {
        //         sqlx::query!(
        //             r#"INSERT INTO turtles
        //                 (id, name, position, orientation, fuel, max_fuel, world)
        //                 VALUES (?,"",?,?,0,0,?);"#,
        //             index,
        //             pos_str,
        //             facing_str,
        //             world
        //         )
        //         .execute(&state.db)
        //         .await
        //         .unwrap();
        //     }
        //     state
        //         .turtle_i_w_map
        //         .write()
        //         .await
        //         .insert(session, (index, world));
        // }
        T2S::Moved(direction) => {
            let world = state.read().await;
            let ident = world
                .get::<ComputerIdent>(entity)
                .context("can't get ident for computer")?;
            let turtle_data = sqlx::query!(
                "SELECT position, orientation FROM turtles WHERE id = ? AND world = ?;",
                ident.id,
                ident.world
            )
            .fetch_one(&state.db)
            .await
            .unwrap();
            let pos = Pos3::from_str_repr(&turtle_data.position).unwrap();
            let orient = Orientation::from_str(&turtle_data.position).unwrap();
            let (new_pos, new_orient) = match direction {
                common::turtle::MoveDirection::Forward => (pos + orient.get_forward_vec(), orient),
                common::turtle::MoveDirection::Back => {
                    (pos + (orient.get_forward_vec() * -1), orient)
                }
                common::turtle::MoveDirection::Up => (pos + Pos3::Y, orient),
                common::turtle::MoveDirection::Down => (pos + Pos3::NEG_Y, orient),
                common::turtle::MoveDirection::Left => {
                    (pos, get_rotated_orientation(orient, TurnDir::Left))
                }
                common::turtle::MoveDirection::Right => {
                    (pos, get_rotated_orientation(orient, TurnDir::Right))
                }
            };
            let new_pos_str = new_pos.to_string_repr();
            let new_orient_str = new_orient.to_string();
            sqlx::query!(
                "UPDATE turtles SET position = ?, orientation = ? WHERE id = ? and world = ?;",
                new_pos_str,
                new_orient_str,
                ident.id,
                ident.world
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        T2S::SetMaxFuel(max) => {
            let world = state.read().await;
            let ident = world
                .get::<ComputerIdent>(entity)
                .context("can't get ident for computer")?;
            sqlx::query!(
                "UPDATE turtles SET max_fuel = ? WHERE id = ? AND world = ?;",
                max,
                ident.id,
                ident.world
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        T2S::SetPos(new_pos) => {
            let world = state.read().await;
            let ident = world
                .get::<ComputerIdent>(entity)
                .context("can't get ident for computer")?;
            let pos_str = new_pos.to_string_repr();
            sqlx::query!(
                "UPDATE turtles SET position = ? WHERE id = ? AND world = ?;",
                pos_str,
                ident.id,
                ident.world
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        T2S::SetOrientation(orient) => {
            let world = state.read().await;
            let ident = world
                .get::<ComputerIdent>(entity)
                .context("can't get ident for computer")?;
            let orient_str = orient.to_string();
            sqlx::query!(
                "UPDATE turtles SET orientation = ? WHERE id = ? AND world = ?;",
                orient_str,
                ident.id,
                ident.world
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        T2S::UpdateSlotContents { slot, contents } => {}
        T2S::UpdateFuel(fuel) => {
            let world = state.read().await;
            let ident = world
                .get::<ComputerIdent>(entity)
                .context("can't get ident for computer")?;
            sqlx::query!(
                "UPDATE turtles SET fuel = ? WHERE id = ? AND world = ?;",
                fuel,
                ident.id,
                ident.world
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        T2S::UpdateBlocks { up, down, front } => {
            async fn write_block(pos: Pos3, ident: Option<String>, world: &str, db: &SqlitePool) {
                let chunk_key = chunk_pos_to_chunk_key(&get_chunk_containing_block(&pos));
                let pos_str = pos.to_string_repr();
                let is_air = ident.is_none();
                let ident = ident.unwrap_or_default();
                sqlx::query!(
                "INSERT OR REPLACE INTO blocks (chunk_key,is_air,id,world,world_pos) VALUES (?,?,?,?,?);",
                chunk_key,
                is_air,
                ident,
                world,
                pos_str,
            )
            .execute(db)
            .await
            .unwrap();
            }
            let world = state.read().await;
            let ident = world
                .get::<ComputerIdent>(entity)
                .context("can't get ident for computer")?;
            let turtle_data = sqlx::query!(
                "SELECT position, orientation FROM turtles WHERE id = ? AND world = ?;",
                ident.id,
                ident.world
            )
            .fetch_one(&state.db)
            .await
            .unwrap();
            let pos = Pos3::from_str_repr(&turtle_data.position).unwrap();
            let orient = Orientation::from_str(&turtle_data.position).unwrap();
            let pos_up = pos + Pos3::Y;
            let pos_down = pos + Pos3::NEG_Y;
            let pos_front = pos + orient.get_forward_vec();
            write_block(pos_up, up, &ident.world, &state.db).await;
            write_block(pos_down, down, &ident.world, &state.db).await;
            write_block(pos_front, front, &ident.world, &state.db).await;
        }
        T2S::ComputerToServer(packet) => {
            return computers::handle_computer_packet(packet, state, entity).await
        }
    };
    Ok(())
}

use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut, Component)]
pub struct TurtleWsSender(SplitSink<WebSocket, Message>);

// pub(crate) trait TurtleWsSenderLockExt {
//     async fn send_packet(&self, session: &Session, packet: &S2TPacket);
// }
//
// impl TurtleWsSenderLockExt for RwLock<SessionMap<TurtleWsSender>> {
//     async fn send_packet(&self, session: &Session, packet: &S2TPacket) {
//         self.write()
//             .await
//             .get_mut(session)
//             .unwrap()
//             .send(packet)
//             .await;
//     }
// }

impl TurtleWsSender {
    pub async fn send(&mut self, packet: &S2TPacket) {
        self.0
            .send(Message::Text(serde_json::to_string(packet).unwrap()))
            .await
            .unwrap();
    }
}

// trait IndexWorldMapExt {
//     async fn get<'a>(
//         &'a self,
//         session: &Session,
//     ) -> color_eyre::Result<RwLockReadGuard<'a, (i32, String)>>;
// }
// impl IndexWorldMapExt for RwLock<SessionMap<(i32, String)>> {
//     async fn get<'a>(
//         &'a self,
//         session: &Session,
//     ) -> color_eyre::Result<RwLockReadGuard<'a, (i32, String)>> {
//         tokio::sync::RwLockReadGuard::<'_, SessionMap<(i32, std::string::String)>>::try_map(
//             self.read().await,
//             |w| w.get(session),
//         )
//         .ok()
//         .ok_or_eyre("Session not in turtle_index_world_map!")
//     }
// }
