use bevy::{log::prelude::*, prelude::*};
use common::client_packets::{C2SPackets, S2CPackets};
use crossbeam_channel::{unbounded, Receiver, Sender};
use futures_util::{stream::{SplitSink, SplitStream}, SinkExt, StreamExt};
use gloo::net::websocket::{futures::WebSocket, Message};
use gloo::render::{request_animation_frame, AnimationFrame};
use serde_json::{from_str, to_string};
use std::{rc::Rc, sync::{Mutex, Arc}};
use wasm_bindgen_futures::spawn_local;

pub struct WS;

impl Plugin for WS {
    #[no_mangle]
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
}
impl Drop for WsCommunicator {
    fn drop(&mut self) {}
}
#[no_mangle]
async fn read(
    mut ws_tx: SplitSink<WebSocket, Message>,
    c2s_rx: Receiver<C2SPackets>,
    old: Rc<Mutex<Option<AnimationFrame>>>,
) {
    match c2s_rx.try_recv() {
        Ok(w) => {
            ws_tx
                .send(Message::Text(to_string(&w).unwrap()))
                .await
                .unwrap();
            info!("message send");
        }
        Err(err) => match err {
            crossbeam_channel::TryRecvError::Empty => (),
            crossbeam_channel::TryRecvError::Disconnected => {
                error!("ws closed");
            }
        },
    };
    let w = old.clone();
    let ran = request_animation_frame(move |_| spawn_local(read(ws_tx, c2s_rx, w)));
    old.lock().unwrap().replace(ran);
}
impl WsCommunicator {
    #[no_mangle]
    pub fn init(ip: &str) -> Self {
        let ws = WebSocket::open(ip).unwrap();
        let (ws_tx, mut ws_rx) = ws.split();

        let (s2c_tx, s2c_rx) = unbounded::<S2CPackets>();
        let (c2s_tx, c2s_rx) = unbounded::<C2SPackets>();
        spawn_local(async move {
            // let mut ind = 0;
            loop {
                // info!("Pre: {ind}");
                // Sometimes just doesn't recive messages?! so yeah won't fix that one!
                let e = ws_rx.next().await;
                // info!("Crazy? ind: {ind}");
                // ind += 1;
                match e {
                    Some(Ok(Message::Text(msg))) => {
                        info!("message!");
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

        spawn_local(read(ws_tx, c2s_rx, Rc::new(Mutex::new(None))));

        Self {
            from_server: s2c_rx,
            to_server: c2s_tx,
        }
    }
}

#[no_mangle]
#[allow(dead_code)]
fn test_ws(mut read: EventReader<S2CPackets>) {
    for p in read.read() {
        info!("{:#?}", p)
    }
}

#[no_mangle]
pub fn run_ws(
    socket: Res<WsCommunicator>,
    mut read: EventReader<C2SPackets>,
    mut write: EventWriter<S2CPackets>,
) {
    for i in socket.from_server.try_iter() {
        write.send(i)
    }
    for i in read.read() {
        _ = socket.to_server.try_send(i.to_owned());
    }
}
