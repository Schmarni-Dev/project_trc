use bevy::{app::AppExit, log::prelude::*, prelude::*};
use common::client_packets::{C2SPackets, S2CPackets};
use crossbeam::channel::{unbounded, Receiver, Sender};
use futures_util::{pin_mut, SinkExt, StreamExt};
use serde_json::{from_str, to_string};
use tokio::runtime::Runtime;
use tokio_tungstenite::connect_async;
use tungstenite::Message;

pub struct WS;

impl Plugin for WS {
    fn build(&self, app: &mut App) {
        let ws_communitcator = WsCommunicator::init("ws://localhost:9001");
        // add things to your app here
        app.add_system(run_ws);
        app.add_system(test_ws);
        app.insert_resource(ws_communitcator);
        app.add_event::<C2SPackets>();
        app.add_event::<S2CPackets>();
    }
}

#[derive(Resource)]
pub struct WsCommunicator {
    to_server: Sender<C2SPackets>,
    from_server: Receiver<S2CPackets>,
    _runtime: Runtime,
}
impl WsCommunicator {
    pub fn init(ip: &str) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .build()
            .unwrap();

        let (mut ws_tx, mut ws_rx) = rt.block_on(async move {
            info!("test: {}", ip);
            let (ws_shit_idc, _) = connect_async(ip).await.unwrap();
            info!("Websocket Connection Established.^^");
            ws_shit_idc.split()
        });

        let (s2c_tx, s2c_rx) = unbounded::<S2CPackets>();
        let (c2s_tx, c2s_rx) = unbounded::<C2SPackets>();
        rt.spawn(async move {
            while let Some(Ok(Message::Text(msg))) = ws_rx.next().await {
                info!("message!");
                if let Ok(msg) = from_str::<S2CPackets>(&msg) {
                    _ = s2c_tx.send(msg);
                }
            }
        });
        rt.spawn(async move {
            while let Some(w) = c2s_rx.iter().next() {
                info!("send message!");
                _ = ws_tx
                    .send(Message::Text(to_string(&w).unwrap()))
                    .await
                    .unwrap();
            }
        });

        Self {
            from_server: s2c_rx,
            to_server: c2s_tx,
            _runtime: rt,
        }
    }
}

fn test_ws(mut read: EventReader<S2CPackets>) {
    for p in read.iter() {
        info!("{:#?}", p)
    }
}

pub fn run_ws(
    socket: Res<WsCommunicator>,
    mut read: EventReader<C2SPackets>,
    mut write: EventWriter<S2CPackets>,
) {
    for i in socket.from_server.try_iter() {
        write.send(i)
    }
    for i in read.iter() {
        _ = socket.to_server.try_send(i.to_owned());
    }
}
