use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use common::{
    client_packets::{C2SPacket, S2CPacket},
    extensions::C2SPacketExtensions,
    remote_control_packets::RcC2SPacket,
};
use derive_more::{Deref, DerefMut};
use futures::{stream::SplitSink, SinkExt as _, StreamExt as _, TryStreamExt as _};
use log::{error, info, warn};
use tokio::sync::RwLock;

use crate::{
    session::{Session, SessionMap},
    AppData,
};

pub async fn client_ws_route(ws: WebSocketUpgrade, state: State<AppData>) -> Response {
    ws.on_upgrade(|s| handle_client_ws(s, state))
}

pub async fn handle_client_ws(socket: WebSocket, state: State<AppData>) {
    let session = state.sessions.acquire_session().await;
    let (writer, mut reader) = socket.split();
    state
        .client_senders
        .write()
        .await
        .insert(session, ClientWsSender(writer));
    state
        .client_current_world
        .write()
        .await
        .insert(session, None);
    while let Ok(msg) = reader.try_next().await {
        let state = state.clone();
        let Some(msg) = msg else {
            warn!("client msg was None");
            continue;
        };
        match msg {
            Message::Text(text) => {
                let Ok(packet) = serde_json::from_str(&text) else {
                    error!("unable to parse client packet: {}", text);
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
                warn!("client ws closed because: {}", w.unwrap().reason);
                break;
            }
        }
    }
    info!("Client disconnected!");
    state.client_senders.write().await.remove(&session);
    state.client_current_world.write().await.remove(&session);
}

async fn handle_packet(
    packet: C2SPacket,
    state: &State<AppData>,
    session: Session,
) -> color_eyre::Result<()> {
    use C2SPacket::ExtensionPacket as Ext;
    use C2SPacketExtensions::PositionTracking as PosT;
    match packet {
        C2SPacket::SwitchWorld(new_world) => {
            state
                .client_current_world
                .write()
                .await
                .insert(session, new_world.into());
        }
        C2SPacket::RequestTurtles => {
            let curr_world_map = state.client_current_world.read().await;
            let world = curr_world_map.get(&session).unwrap();

            if let Some(world) = world.as_ref() {
                state
                    .client_senders
                    .send_packet(
                        &session,
                        &S2CPacket::SetTurtles(common::client_packets::SetTurtlesData {
                            turtles: Vec::new(),
                            world: world.clone(),
                        }),
                    )
                    .await;
            }
        }
        C2SPacket::RequestWorld => {}
        C2SPacket::SendLuaToTurtle { index, world, code } => {}
        C2SPacket::StdInForTurtle {
            index,
            world,
            value,
        } => {}
        Ext(PosT(RcC2SPacket::MoveTurtle {
            index,
            world,
            direction,
        })) => {}
        Ext(PosT(RcC2SPacket::PlaceBlock {
            index,
            world,
            dir,
            text,
        })) => {}
        Ext(PosT(RcC2SPacket::BreakBlock { index, world, dir })) => {}
        Ext(PosT(RcC2SPacket::TurtleSelectSlot { index, world, slot })) => {}
    };
    Ok(())
}

#[derive(Deref, DerefMut)]
pub struct ClientWsSender(SplitSink<WebSocket, Message>);

pub(crate) trait ClientWsSenderLockExt {
    async fn send_packet(&self, session: &Session, packet: &S2CPacket);
}

impl ClientWsSender {
    pub async fn send(&mut self, packet: &S2CPacket) {
        self.0
            .send(Message::Text(serde_json::to_string(packet).unwrap()))
            .await
            .unwrap();
    }
}

impl ClientWsSenderLockExt for RwLock<SessionMap<ClientWsSender>> {
    async fn send_packet(&self, session: &Session, packet: &S2CPacket) {
        self.write()
            .await
            .get_mut(session)
            .unwrap()
            .send(packet)
            .await;
    }
}
