use bevy::{log::prelude::*, prelude::*};
use common::client_packets::{C2SPackets, S2CPackets};
use crossbeam::channel::{unbounded, Receiver, Sender};
use futures_util::{SinkExt, StreamExt};
use serde_json::{from_str, to_string};
use tokio::{runtime::Runtime, task::JoinHandle};
use tokio_tungstenite::connect_async;
use tungstenite::Message;

pub struct WS;

impl Plugin for WS {
    fn build(&self, app: &mut App) {
        let ws_communitcator = WsCommunicator::init("ws://schmerver.mooo.com:9001");
        // add things to your app here
        app.insert_resource(ws_communitcator);
        app.add_systems(Update, run_ws);
        // app.add_systems(Update, test_ws);
        app.add_systems(Startup, run_ws);
        // app.add_systems(Startup, test_ws);
        app.add_event::<C2SPackets>();
        app.add_event::<S2CPackets>();
    }
}

#[derive(Resource)]
pub struct WsCommunicator {
    to_server: Sender<C2SPackets>,
    from_server: Receiver<S2CPackets>,
    _runtime: Runtime,
    join_handles: [JoinHandle<()>; 2],
}
impl Drop for WsCommunicator {
    fn drop(&mut self) {
        for h in &self.join_handles {
            h.abort();
        }
    }
}
impl WsCommunicator {
    pub fn init(ip: &str) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();

        let (mut ws_tx, mut ws_rx) = rt.block_on(async move {
            info!("test: {}", ip);
            let (ws, _) = connect_async(ip).await.unwrap();
            info!("Websocket Connection Established.^^");
            ws.split()
        });

        let (s2c_tx, s2c_rx) = unbounded::<S2CPackets>();
        let (c2s_tx, c2s_rx) = unbounded::<C2SPackets>();
        let ws_read_handle = rt.spawn(async move {
            // let mut ind = 0;
            loop {
                // info!("Pre: {ind}");
                // Sometimes just doesn't recive messages?! so yeah won't fix that one!
                let e = ws_rx.next().await;
                // info!("Crazy? ind: {ind}");
                // ind += 1;
                match e {
                    Some(Ok(Message::Text(msg))) => {
                        // info!("message!");
                        if let Ok(msg) = from_str::<S2CPackets>(&msg) {
                            _ = s2c_tx.send(msg);
                        }
                    }
                    Some(Ok(_fckit)) => {
                        // info!("non text msg {:#?}", fckit);
                    }
                    Some(Err(err)) => {
                        error!("ws error: {err}");
                        break;
                    }
                    None => {
                        error!("ws closed, i think");
                        break;
                    }
                }
            }
        });
        let ws_write_handle = rt.spawn(async move {
            loop {
                match c2s_rx.try_recv() {
                    Ok(w) => {
                        _ = ws_tx
                            .send(Message::Text(to_string(&w).unwrap()))
                            .await
                            .unwrap();
                    }
                    Err(err) => match err {
                        crossbeam::channel::TryRecvError::Empty => (),
                        crossbeam::channel::TryRecvError::Disconnected => {
                            error!("ws closed");
                            break;
                        }
                    },
                }
            }
        });

        Self {
            from_server: s2c_rx,
            to_server: c2s_tx,
            _runtime: rt,
            join_handles: [ws_read_handle, ws_write_handle],
        }
    }
}

#[allow(dead_code)]
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
