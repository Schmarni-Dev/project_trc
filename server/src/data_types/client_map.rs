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
        // Kids, Dont leave a random Debug value in the code you're expecting to work.
        self.0.insert(client.get_index(), client);
        self
    }
    pub async fn broadcast(&mut self, msg: S2CPackets) {
        for (_, c) in &mut self.0 {
            c.send_msg(&msg).await;
        }
    }
    pub async fn send_to(&mut self, msg: S2CPackets, id: &i32) -> Option<()> {
        info!("test: {}", id);
        self.0.get_mut(id)?.send_msg(&msg).await;
        info!("test2: {}", id);
        Some(())
    }
    pub async fn execute_the_client(&mut self, id: &i32) {
        if let Some(client) = self.0.remove(id) {
            client.delete().await
        }
    }
}
