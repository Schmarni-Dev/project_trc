use common::turtle::{Maybe, Orientation};

use common::world_data::Block;
use common::Pos3;

use sqlx::SqlitePool;

use std::str::FromStr;

pub type DB = SqlitePool;

#[derive(Clone, Debug)]
pub(crate) struct DbBlock {
    pub(crate) id: String,
    pub(crate) is_air: bool,
    pub(crate) world: String,
    pub(crate) world_pos: String,
    #[allow(dead_code)]
    pub(crate) chunk_key: i64,
}

impl From<DbBlock> for Block {
    fn from(value: DbBlock) -> Self {
        Self {
            world: value.world,
            id: value.id,
            pos: parse_pos3_from_db_str(&value.world_pos)
                .expect("DB should really have a valid pos string"),
            is_air: value.is_air,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DbTurtle {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) position: String,
    pub(crate) orientation: String,
    pub(crate) fuel: i64,
    pub(crate) max_fuel: i64,
    pub(crate) world: String,
}

impl From<DbTurtle> for common::turtle::Turtle {
    fn from(value: DbTurtle) -> Self {
        Self {
            index: value
                .id
                .try_into()
                .expect("should fit since everywhere is i32, only db is i64"),
            name: value.name,
            inventory: Maybe::None,
            position: parse_pos3_from_db_str(&value.position)
                .expect("DB should really have a valid pos string"),
            orientation: Orientation::from_str(&value.orientation)
                .expect("DB should really have a valid orientation string"),
            fuel: value
                .fuel
                .try_into()
                .expect("should fit since everywhere is i32, only db is i64"),
            max_fuel: value
                .max_fuel
                .try_into()
                .expect("should fit since everywhere is i32, only db is i64"),
            is_online: false,
            world: value.world,
        }
    }
}

pub fn pos_to_db_pos(pos: &Pos3) -> String {
    format!("{};{};{}", pos.x, pos.y, pos.z)
}

pub const fn get_chunk_key_mask(i: usize) -> u32 {
    match i {
        0 => 0b001001001001001001001001001001001,
        1 => 0b010010010010010010010010010010010,
        2 => 0b001001001001001001001001001001001,
        _ => 0,
    }
}

pub const fn pos_to_key(pos: &Pos3) -> u32 {
    let x = pos.x.unsigned_abs() & get_chunk_key_mask(0);
    let y = pos.y.unsigned_abs() & get_chunk_key_mask(1);
    let z = pos.z.unsigned_abs() & get_chunk_key_mask(2);
    x | y | z
}

fn parse_pos3_from_db_str(str: &str) -> anyhow::Result<Pos3> {
    let poses = str.split(';');
    let mut poses = poses.map(|v| v.parse::<i32>());
    let x = poses
        .next()
        .ok_or(anyhow::anyhow!("could not parse first int"))??;
    let y = poses
        .next()
        .ok_or(anyhow::anyhow!("could not parse second int"))??;
    let z = poses
        .next()
        .ok_or(anyhow::anyhow!("could not parse third int"))??;
    Ok(Pos3::new(x, y, z))
}
