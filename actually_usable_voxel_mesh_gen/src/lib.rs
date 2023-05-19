use data::{BlockFaceData, ChunkData, Side};
use rayon::prelude::*;
use stuff::get_vertecies_from_side;
pub mod data;
mod stuff;
mod util;
use common::Pos3;

pub fn generate_mesh_for_chunk(chunk_data: &impl ChunkData) -> Vec<BlockFaceData> {
    let size = chunk_data.get_chunk_size();
    let mut plane = Vec::<(i32, i32)>::new();
    (0..size).for_each(|x| {
        (0..size).for_each(|z| plane.push((x, z)));
    });
    (0..size)
        .into_par_iter()
        .map(|y| {
            plane.par_iter().map(move |(x, z)| {
                get_face_data_for_block(
                    chunk_data,
                    Pos3 {
                        x: x.to_owned(),
                        y,
                        z: z.to_owned(),
                    },
                )
                .into_par_iter()
            })
        })
        .flatten()
        .flatten()
        .collect()
}
#[allow(unused)]
fn get_face_data_for_block(chunk_data: &impl ChunkData, pos: Pos3) -> Vec<BlockFaceData> {
    let mut out = Vec::new();
    if !chunk_data.does_block_exits(&pos) {
        return out;
    }
    let color: [u8; 3] = util::string_to_color(&chunk_data.get_color_seed_for_block(&pos))[0..3]
        .try_into()
        .unwrap();
    for side in [
        Side::PosX,
        Side::NegX,
        Side::PosY,
        Side::NegY,
        Side::PosZ,
        Side::NegZ,
    ] {
        if !chunk_data.has_neighbour(&pos, &side) {
            let vertecies = get_vertecies_from_side(&side)
                .into_iter()
                .map(|p| p + &pos)
                .collect::<Vec<Pos3>>()
                .try_into()
                .unwrap();
            let normal = side.side_to_rel_pos();
            let color = color.clone();
            let data = BlockFaceData {
                color,
                normal,
                vertecies,
            };
            out.push(data);
        }
    }
    out
}
