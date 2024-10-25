use std::collections::HashMap;

use super::server_client::ServerClient;
use common::client_packets::S2CPackets;

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
        for c in self.0.values_mut() {
            c.send_msg(&msg).await;
        }
    }
    pub async fn send_to(&mut self, msg: S2CPackets, id: &i32) -> Option<()> {
        self.0.get_mut(id)?.send_msg(&msg).await;
        Some(())
    }
    pub async fn execute_the_client(&mut self, id: &i32) {
        if let Some(client) = self.0.remove(id) {
            client.delete().await
        }
    }
}

impl Default for ClientMap {
    fn default() -> Self {
        Self::new()
    }
}
