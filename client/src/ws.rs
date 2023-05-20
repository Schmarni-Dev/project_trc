use bevy::{log::prelude::*, prelude::*};
use common::client_packets::{C2SPackets, S2CPackets};
use crossbeam::channel::{unbounded, Receiver, Sender};
use futures_util::{SinkExt, StreamExt};
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

        // rt.spawn(async move {
        //     println!("WS MSG: {}", "txt2");
        //     while let Some(Ok(Message::Text(txt))) = ws_rx.next().await {
        //         println!("WS MSG: {}", txt);
        //         if let Ok(msg) = from_str::<S2CPackets>(&txt) {
        //             _ = s2c_tx.send(msg);
        //         }
        //     }
        //     println!("WS MSG: {}", "txt");
        // });
        // rt.spawn(async move {
        //     _ = ws_tx
        //         .send(Message::Text(
        //             to_string(&C2SPackets::RequestTurtles).unwrap(),
        //         ))
        //         .await
        //         .unwrap();
        //     while let Some(w) = c2s_rx.iter().next() {
        //         _ = ws_tx.send(Message::Text(to_string(&w).unwrap())).await;
        //     }
        // });

        Self {
            from_server: s2c_rx,
            to_server: c2s_tx,
            _runtime: rt,
        }
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
