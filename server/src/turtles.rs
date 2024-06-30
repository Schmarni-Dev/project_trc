use std::str::FromStr;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use color_eyre::eyre::OptionExt;
use common::{
    turtle::{get_rotated_orientation, Orientation, TurnDir},
    turtle_packets::{S2TPacket, SetupInfoData, T2SPackets},
    world_data::{chunk_pos_to_chunk_key, get_chunk_containing_block},
    Pos3,
};
use futures::{stream::SplitSink, Sink, SinkExt, StreamExt, TryStreamExt};
use log::{error, info, warn};
use tokio::sync::{RwLock, RwLockReadGuard};

use crate::{
    session::{Session, SessionMap},
    AppData,
};

pub async fn turtle_ws_route(ws: WebSocketUpgrade, state: State<AppData>) -> Response {
    ws.on_upgrade(|s| handle_turtle_ws(s, state))
}

pub async fn handle_turtle_ws(socket: WebSocket, state: State<AppData>) {
    let session = state.sessions.acquire_session().await;
    let (writer, mut reader) = socket.split();
    state
        .turtle_senders
        .write()
        .await
        .insert(session, TurtleWsSender(writer));
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
                if let Err(err) = handle_packet(packet, &state, session).await {
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
    state.turtle_senders.write().await.remove(&session);
    state.turtle_i_w_map.write().await.remove(&session);
}

async fn handle_packet(
    packet: T2SPackets,
    state: &State<AppData>,
    session: Session,
) -> color_eyre::Result<()> {
    match packet {
        T2SPackets::SetupInfo(SetupInfoData {
            facing,
            position,
            index,
            world,
        }) => {
            let exits = sqlx::query!(
                "SELECT TRUE as stored FROM turtles WHERE id = ? AND world = ?;",
                index,
                world
            )
            .fetch_one(&state.db)
            .await
            .unwrap();
            let facing_str = facing.to_string();
            let pos_str = position.to_string_repr();
            if exits.stored != 0 {
                sqlx::query!(
                    "UPDATE turtles SET orientation = ?, position = ? WHERE id = ? AND world = ?;",
                    facing_str,
                    pos_str,
                    index,
                    world
                )
                .execute(&state.db)
                .await
                .unwrap();
            } else {
                sqlx::query!(
                    r#"INSERT INTO turtles 
                        (id, name, position, orientation, fuel, max_fuel, world) 
                        VALUES (?,"",?,?,0,0,?);"#,
                    index,
                    pos_str,
                    facing_str,
                    world
                )
                .execute(&state.db)
                .await
                .unwrap();
            }
            state
                .turtle_i_w_map
                .write()
                .await
                .insert(session, (index, world));
        }
        T2SPackets::Moved { direction } => {
            let t = state.turtle_i_w_map.get(&session).await?;
            let turtle_data = sqlx::query!(
                "SELECT position, orientation FROM turtles WHERE id = ? AND world = ?;",
                t.0,
                t.1
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
                t.0,
                t.1
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        T2SPackets::SetMaxFuel(max) => {
            let t = state.turtle_i_w_map.get(&session).await?;
            sqlx::query!(
                "UPDATE turtles SET max_fuel = ? WHERE id = ? AND world = ?;",
                max,
                t.0,
                t.1
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        T2SPackets::SetPos(new_pos) => {
            let t = state.turtle_i_w_map.get(&session).await?;
            let pos_str = new_pos.to_string_repr();
            sqlx::query!(
                "UPDATE turtles SET position = ? WHERE id = ? AND world = ?;",
                pos_str,
                t.0,
                t.1
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        T2SPackets::SetOrientation(orient) => {
            let t = state.turtle_i_w_map.get(&session).await?;
            let orient_str = orient.to_string();
            sqlx::query!(
                "UPDATE turtles SET orientation = ? WHERE id = ? AND world = ?;",
                orient_str,
                t.0,
                t.1
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        T2SPackets::ChangeWorld(new_world) => {
            let t = state.turtle_i_w_map.get(&session).await?;
            sqlx::query!(
                "UPDATE turtles SET world = ? WHERE id = ? AND world = ?;",
                new_world,
                t.0,
                t.1
            )
            .execute(&state.db)
            .await
            .unwrap();
            drop(t);
            state
                .turtle_i_w_map
                .write()
                .await
                .get_mut(&session)
                .ok_or_eyre("no data for session in index_world_map")?
                .1 = new_world;
        }
        T2SPackets::InventoryUpdate(_) => {}
        T2SPackets::SetName(name) => {
            let t = state.turtle_i_w_map.get(&session).await?;
            sqlx::query!(
                "UPDATE turtles SET name = ? WHERE id = ? AND world = ?;",
                name,
                t.0,
                t.1
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        T2SPackets::FuelUpdate(fuel) => {
            let t = state.turtle_i_w_map.get(&session).await?;
            sqlx::query!(
                "UPDATE turtles SET fuel = ? WHERE id = ? AND world = ?;",
                fuel,
                t.0,
                t.1
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        T2SPackets::Blocks { up, down, front } => {
            let t = state.turtle_i_w_map.get(&session).await?;
            let up: Option<String> = up.into();
            let down: Option<String> = down.into();
            let front: Option<String> = front.into();
            let turtle_data = sqlx::query!(
                "SELECT position, orientation FROM turtles WHERE id = ? AND world = ?;",
                t.0,
                t.1
            )
            .fetch_one(&state.db)
            .await
            .unwrap();
            let pos = Pos3::from_str_repr(&turtle_data.position).unwrap();
            let orient = Orientation::from_str(&turtle_data.position).unwrap();
            let pos_up = pos + Pos3::Y;
            let pos_down = pos + Pos3::NEG_Y;
            let pos_front = pos + orient.get_forward_vec();
            let chunk_key_up = chunk_pos_to_chunk_key(&get_chunk_containing_block(&pos_up));
            let chunk_key_down = chunk_pos_to_chunk_key(&get_chunk_containing_block(&pos_down));
            let chunk_key_front = chunk_pos_to_chunk_key(&get_chunk_containing_block(&pos_front));
            let pos_str_up = pos_up.to_string_repr();
            let pos_str_down = pos_down.to_string_repr();
            let pos_str_front = pos_front.to_string_repr();
            let is_air_up = up.is_none();
            let is_air_down = down.is_none();
            let is_air_front = front.is_none();
            let ident_up = up.unwrap_or_default();
            let ident_down = down.unwrap_or_default();
            let ident_front = front.unwrap_or_default();
            sqlx::query!(
                "INSERT OR REPLACE INTO blocks (chunk_key,is_air,id,world,world_pos) VALUES (?,?,?,?,?);",
                chunk_key_up,
                is_air_up,
                ident_up,
                t.1,
                pos_str_up,
            )
            .execute(&state.db)
            .await
            .unwrap();
            sqlx::query!(
                "INSERT OR REPLACE INTO blocks (chunk_key,is_air,id,world,world_pos) VALUES (?,?,?,?,?);",
                chunk_key_down,
                is_air_down,
                ident_down,
                t.1,
                pos_str_down,
            )
            .execute(&state.db)
            .await
            .unwrap();
            sqlx::query!(
                "INSERT OR REPLACE INTO blocks (chunk_key,is_air,id,world,world_pos) VALUES (?,?,?,?,?);",
                chunk_key_front,
                is_air_front,
                ident_front,
                t.1,
                pos_str_front,
            )
            .execute(&state.db)
            .await
            .unwrap();
        }
        T2SPackets::Executables(_) => {}
        T2SPackets::Ping => {}
        T2SPackets::StdOut(_) => {}
        T2SPackets::ExtPacket(_) => {}
    };
    Ok(())
}

use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut)]
pub struct TurtleWsSender(SplitSink<WebSocket, Message>);

pub(crate) trait TurtleWsSenderLockExt {
    async fn send_packet(&self, session: &Session, packet: &S2TPacket);
}

impl TurtleWsSenderLockExt for RwLock<SessionMap<TurtleWsSender>> {
    async fn send_packet(&self, session: &Session, packet: &S2TPacket) {
        self.write()
            .await
            .get_mut(session)
            .unwrap()
            .send(packet)
            .await;
    }
}

impl TurtleWsSender {
    pub async fn send(&mut self, packet: &S2TPacket) {
        self.0
            .send(Message::Text(serde_json::to_string(packet).unwrap()))
            .await
            .unwrap();
    }
}

trait IndexWorldMapExt {
    async fn get<'a>(
        &'a self,
        session: &Session,
    ) -> color_eyre::Result<RwLockReadGuard<'a, (i32, String)>>;
}
impl IndexWorldMapExt for RwLock<SessionMap<(i32, String)>> {
    async fn get<'a>(
        &'a self,
        session: &Session,
    ) -> color_eyre::Result<RwLockReadGuard<'a, (i32, String)>> {
        tokio::sync::RwLockReadGuard::<'_, SessionMap<(i32, std::string::String)>>::try_map(
            self.read().await,
            |w| w.get(session),
        )
        .ok()
        .ok_or_eyre("Session not in turtle_index_world_map!")
    }
}
