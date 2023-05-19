// use std::net::SocketAddr;

// use tungstenite::Message;

// pub fn broadcast_all(peer_map: &crate::PeerMap, msg: &Message) {
//     let peers = peer_map.lock().unwrap();

//     // We want to broadcast the message to everyone including ourselves.
//     let broadcast_recipients = peers
//         .iter()
//         // .filter(|(peer_addr, _)| peer_addr != &&addr)
//         .map(|(_, ws_sink)| ws_sink);

//     for recp in broadcast_recipients {
//         recp.unbounded_send(msg.clone()).unwrap();
//     }
// }

// pub fn broadcast_others(peer_map: &crate::PeerMap, message: &Message, except_addr: &SocketAddr) {
//     let peers = peer_map.lock().unwrap();

//     // We want to broadcast the message to everyone including ourselves.
//     let broadcast_recipients = peers
//         .iter()
//         .filter(|(peer_addr, _)| peer_addr != &except_addr)
//         .map(|(_, ws_sink)| ws_sink);

//     for recp in broadcast_recipients {
//         recp.unbounded_send(message.clone()).unwrap();
//     }
// }

// pub fn send_to(peer_map: &crate::PeerMap, msg: &Message, target_addr: &SocketAddr) {
//     let peers = peer_map.lock().unwrap();

//     // We want to broadcast the message to everyone including ourselves.
//     let broadcast_recipients = peers
//         .iter()
//         .filter(|(peer_addr, _)| peer_addr == &target_addr)
//         .map(|(_, ws_sink)| ws_sink);

//     for recp in broadcast_recipients {
//         recp.unbounded_send(msg.clone()).unwrap();
//     }
// }
