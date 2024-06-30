use std::ops::Deref;

use bevy::prelude::*;
use common::Pos3;

// use crate::idk::ClientChunk;


// #[derive(Component)]
// pub struct ChunkInstance {
//     chunk_pos: Pos3,
//     chunk_data: ClientChunk,
//     pub setup: bool,
// }
//
// impl ChunkInstance {
//     pub fn new(chunk_pos: Pos3, chunk_data: ClientChunk) -> ChunkInstance {
//         ChunkInstance {
//             chunk_pos,
//             chunk_data,
//             setup: true,
//         }
//     }
//     pub fn get_chunk_pos(&self) -> &Pos3 {
//         &self.chunk_pos
//     }
//     pub fn inner_mut(&mut self) -> &mut ClientChunk {
//         &mut self.chunk_data
//     }
// }
//
// impl Deref for ChunkInstance {
//     type Target = ClientChunk;
//     fn deref(&self) -> &Self::Target {
//         &self.chunk_data
//     }
// }
