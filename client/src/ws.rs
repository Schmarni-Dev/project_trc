use bevy::prelude::*;
use common::client_packets::{C2SPackets, MovedTurtleData, S2CPackets};
use crossbeam::channel::*;
use serde_json::{from_str, to_string_pretty};
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

pub fn setup_ws(mut commands: Commands) {
    let (mut socket, response) =
        connect(Url::parse("ws://localhost:9001").unwrap()).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }
    let (ws_recv_tx, ws_recv_rx) = unbounded::<S2CPackets>();
    let (ws_send_tx, ws_send_rx) = unbounded::<C2SPackets>();
    commands.insert_resource(WsSendChannel(ws_send_tx));
    commands.insert_resource(WsRecvChannel(ws_recv_rx));
    // socket
    //     .write_message(Message::Text("Hello WebSocket".into()))
    // .unwrap();
    std::thread::spawn(move || loop {
        for msg in ws_send_rx.try_iter() {
            _ = socket.write_message(Message::Text(to_string_pretty(&msg).unwrap()));
        }
        if let Ok(Message::Text(msg)) = socket.read_message() {
            println!("Received: {}", msg);
            if let Ok(packet) = from_str::<S2CPackets>(&msg) {
                _ = ws_recv_tx.send(packet);
            };
        };
    });
}

pub fn read_ws_messages(receiver: Res<WsRecvChannel>, mut events: EventWriter<WsEvents>) {
    for from_stream in receiver.try_iter() {
        match from_stream {
            S2CPackets::MovedTurtle(data) => events.send(WsEvents::TurtleMoved(data)),
            S2CPackets::RequestedTurtles(t) => {}
        }
    }
}
