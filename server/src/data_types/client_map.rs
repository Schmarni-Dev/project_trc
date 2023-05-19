use std::collections::HashMap;

use super::server_client::ServerClient;
use common::client_packets::S2CPackets;
use log::info;

pub struct DosentExist;

pub struct ClientMap(HashMap<i32, ServerClient>);

impl ClientMap {
    pub fn new() -> ClientMap {
        ClientMap(HashMap::new())
    }
    pub fn push(&mut self, client: ServerClient) -> &mut Self {
        self.0.insert(1, client);
        self
    }
    pub async fn broadcast(&mut self, msg: S2CPackets) {
        info!("test");
        for (_, c) in &mut self.0 {
            c.send_msg(&msg).await;
        }
    }
    pub async fn send_to(&mut self, msg: S2CPackets, id: &i32) -> Option<()> {
        self.0.get_mut(id)?.send_msg(&msg).await;
        Some(())
    }
}
