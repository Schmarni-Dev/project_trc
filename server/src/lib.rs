pub mod connection_manager;
pub mod data_types;
pub mod db;
// pub mod fake;
pub mod send_util;
// mod turtle;
pub mod handle_turtles;
pub mod util;

use futures_channel::mpsc::UnboundedSender;
use tungstenite::protocol::Message;

pub type Tx = UnboundedSender<Message>;
pub mod handle_clients;
