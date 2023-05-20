use bevy::prelude::*;
use common::client_packets::{C2SPackets, MovedTurtleData, S2CPackets};
use crossbeam::channel::*;
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use serde_json::{from_str, to_string_pretty};
use tokio::runtime::Runtime;
use tokio_tungstenite::connect_async;
use tungstenite::*;
use url::Url;
#[derive(Resource, Deref, DerefMut)]
/// DON'T USE THIS USE THE EVENTS!
pub struct WsSendChannel(Sender<C2SPackets>);
#[derive(Resource, Deref, DerefMut)]
/// DON'T USE THIS USE THE EVENTS!
pub struct WsRecvChannel(Receiver<S2CPackets>);

pub enum WsEvents {
    TurtleMoved(MovedTurtleData),
}

pub fn setup_ws(mut commands: Commands, mut event: EventWriter<C2SPackets>) {
    let (ws_recv_tx, ws_recv_rx) = unbounded::<S2CPackets>();
    let (ws_send_tx, ws_send_rx) = unbounded::<C2SPackets>();
    commands.insert_resource(WsSendChannel(ws_send_tx));
    commands.insert_resource(WsRecvChannel(ws_recv_rx));
    let rt = Runtime::new().unwrap();

    // Spawn the root task
    rt.block_on(async {
        tokio::spawn(async move {
            let (mut socket, response) = connect_async(Url::parse("ws://localhost:9001").unwrap())
                .await
                .expect("Can't connect");

            println!("Connected to the server");
            println!("Response HTTP code: {}", response.status());
            println!("Response contains the following headers:");
            for (ref header, _value) in response.headers() {
                println!("* {}", header);
            }
            let (mut write, mut read) = socket.split();
            tokio::spawn(async move {
                while let Ok(Some(Message::Text(msg))) = read.try_next().await {
                    println!("Received: {}", msg);
                    if let Ok(packet) = from_str::<S2CPackets>(&msg) {
                        _ = ws_recv_tx.send(packet);
                    };
                }
            });
            while let Ok(msg) = ws_send_rx.recv() {
                println!("wsSendRecived!");
                _ = write
                    .send(Message::Text(to_string_pretty(&msg).unwrap()))
                    .await;
            }
        });
        println!("test async");
    });
    event.send(C2SPackets::RequestTurtles);
}

pub fn read_ws_messages(receiver: Res<WsRecvChannel>, mut events: EventWriter<WsEvents>) {
    for from_stream in receiver.try_iter() {
        match from_stream {
            S2CPackets::MovedTurtle(data) => events.send(WsEvents::TurtleMoved(data)),
            S2CPackets::RequestedTurtles(t) => {}
        }
    }
}
pub fn write_ws_messages(sender: Res<WsSendChannel>, mut events: EventReader<C2SPackets>) {
    for e in events.iter() {
        println!("test");
        let e = e.to_owned();
        _ = sender.send(e);
    }
}
