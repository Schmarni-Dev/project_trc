use common::turtle::{Orientation, Turtle, TurtleIndexType};

use common::world_data::{get_chunk_containing_block, Block, World};
use common::Pos3;
use libsql_client::{args, Client, ResultSet, Row};
use libsql_client::{Statement as S, Value};

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub enum DBDataTypes {
    Turtles(HashMap<i32, Turtle>),
    World(World),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DBTables {
    Turtles,
    World,
}

type DBType = HashMap<DBTables, DBDataTypes>;

pub struct DB {
    pub client: Client,
}

fn parse_turtle_instance_from_row(row: &Row) -> anyhow::Result<Turtle> {
    let index: i32 = parse_i64_from_row(&row, "id")?.try_into()?;
    let name = parse_string_from_row(&row, "name")?;
    let inv = serde_json::from_str(&parse_string_from_row(&row, "inventory")?)?;
    let pos = parse_pos3_from_row(&row, "position")?;
    let orient = Orientation::from_str(&parse_string_from_row(&row, "orientation")?)
        .map_err(|e| anyhow::anyhow!(e))?;
    let fuel: i32 = parse_i64_from_row(&row, "fuel")?.try_into()?;
    let max_fuel: i32 = parse_i64_from_row(&row, "max_fuel")?.try_into()?;
    let world = parse_string_from_row(&row, "world")?;
    Ok(Turtle::new(
        index, name, inv, pos, orient, fuel, max_fuel, false, world,
    ))
}
fn parse_block_instance_from_row(row: &Row) -> anyhow::Result<Block> {
    let id = parse_string_from_row(&row, "id")?;
    let world = parse_string_from_row(&row, "world")?;
    let pos = parse_pos3_from_row(&row, "world_pos")?;
    let is_air = parse_bool_from_row(&row, "is_air")?;
    Ok(Block {
        world,
        id,
        pos,
        is_air,
    })
}

pub fn pos_to_db_pos(pos: &Pos3) -> Arc<str> {
    Arc::from(format!("{};{};{}", pos.x, pos.y, pos.z))
}

const fn get_chunk_key_mask(i: usize) -> u32 {
    match i {
        0 => 0b001001001001001001001001001001001,
        1 => 0b010010010010010010010010010010010,
        2 => 0b001001001001001001001001001001001,
        _ => 0,
    }
}

fn pos_to_key(pos: &Pos3) -> u32 {
    let x = pos.x.unsigned_abs() & get_chunk_key_mask(0);
    let y = pos.y.unsigned_abs() & get_chunk_key_mask(1);
    let z = pos.z.unsigned_abs() & get_chunk_key_mask(2);
    x | y | z
}

#[derive(Debug)]
pub enum DBError {
    NotFound,
    Error(anyhow::Error),
}

impl From<anyhow::Error> for DBError {
    fn from(value: anyhow::Error) -> Self {
        DBError::Error(value)
    }
}

impl DB {
    pub async fn new() -> anyhow::Result<DB> {
        let db = libsql_client::Client::from_env().await?;
        db.execute(
            "
        CREATE TABLE IF NOT EXISTS turtles (
            id INTEGER NOT NULL,
            name TEXT NOT NULL,
            inventory TEXT NOT NULL,
            position TEXT NOT NULL,
            orientation TEXT NOT NULL,
            fuel INTEGER NOT NULL,
            max_fuel INTEGER NOT NULL,
            world TEXT NOT NULL,
            PRIMARY KEY (world,id),
            FOREIGN KEY (world)
                REFERENCES worlds (name)
        );",
        )
        .await?;
        db.execute(
            "
        CREATE TABLE IF NOT EXISTS worlds (
            name TEXT NOT NULL UNIQUE PRIMARY KEY
        );",
        )
        .await?;
        db.execute(
            "
        CREATE TABLE IF NOT EXISTS blocks (
            chunk_key INTEGER NOT NULL,
            id TEXT NOT NULL,
            world TEXT NOT NULL,
            world_pos TEXT NOT NULL,
            is_air BOOLEAN NOT NULL,
            PRIMARY KEY (world,world_pos),
            FOREIGN KEY (world)
                REFERENCES worlds (name)
        );",
        )
        .await?;
        Ok(DB { client: db })
    }

    async fn insert_turtle(&self, turtle: &Turtle) -> anyhow::Result<()> {
        self.exec(
            "INSERT INTO turtles VALUES (?,?,?,?,?,?,?,?)",
            args!(
                turtle.index,
                &turtle.name,
                serde_json::to_string(&turtle.inventory)?,
                *pos_to_db_pos(&turtle.position),
                turtle.orientation.to_string(),
                turtle.fuel,
                turtle.max_fuel,
                &turtle.world
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn get_dummy_turtle(
        &self,
        index: TurtleIndexType,
        world: String,
        pos: Pos3,
        orientation: Orientation,
    ) -> anyhow::Result<Turtle> {
        let t = Turtle::new_dummy(index, world, pos, orientation);
        self.insert_turtle(&t).await?;
        Ok(t)
    }

    pub async fn get_worlds(&self) -> anyhow::Result<Vec<String>> {
        Ok(self
            .client
            .execute("SELECT name FROM worlds;")
            .await?
            .rows
            .into_iter()
            .flat_map(|row| row.values)
            .filter_map(|value| match value {
                Value::Text { value } => Some(value),
                _ => None,
            })
            .collect())
    }

    pub async fn create_world(&self, name: &str) -> anyhow::Result<()> {
        self.exec("INSERT OR IGNORE INTO worlds VALUES (?);", args!(name))
            .await?;
        Ok(())
    }

    pub async fn get_world(&self, name: &str) -> anyhow::Result<World> {
        let mut world = World::new(name);
        let blocks = self
            .exec("SELECT * FROM blocks WHERE world = ?;", args!(name))
            .await?
            .rows
            .iter()
            .map(|row| parse_block_instance_from_row(row))
            .collect::<anyhow::Result<Vec<_>>>()?;
        for b in blocks.into_iter() {
            world.set_block(b);
        }
        Ok(world)
    }

    pub async fn get_turtle(
        &self,
        index: TurtleIndexType,
        world_name: &str,
    ) -> Result<Turtle, DBError> {
        let w = self
            .client
            .execute(S::with_args(
                "SELECT * FROM turtles WHERE id = ? AND world = ?; ",
                args!(index, world_name),
            ))
            .await?;
        let row = w.rows.first().ok_or(DBError::NotFound)?;
        let turtle = parse_turtle_instance_from_row(&row)?;

        Ok(turtle)
    }

    pub async fn exec(
        &self,
        query: impl Into<String>,
        params: &[impl Into<Value> + Clone],
    ) -> anyhow::Result<ResultSet> {
        self.client.execute(S::with_args(query, params)).await
    }

    pub async fn set_block(&self, block: &Block) -> anyhow::Result<()> {
        self.exec(
            "INSERT OR REPLACE INTO blocks VALUES (?,?,?,?,?)",
            args!(
                pos_to_key(&get_chunk_containing_block(&block.pos)),
                &block.id,
                &block.world,
                *pos_to_db_pos(&block.pos),
                match &block.is_air {
                    true => 1,
                    false => 0,
                }
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn get_turtles(&self, world_name: &str) -> anyhow::Result<Vec<Turtle>> {
        self.client
            .execute(S::with_args(
                "SELECT * FROM turtles WHERE world = ?;",
                args!(world_name),
            ))
            .await?
            .rows
            .iter()
            .map(|row| parse_turtle_instance_from_row(row))
            .collect::<anyhow::Result<Vec<_>>>()
    }

    /// Unsure
    pub fn turtle_exists(&self, _index: TurtleIndexType) -> bool {
        todo!()
    }
}

fn parse_i64_from_row(row: &Row, name: &str) -> anyhow::Result<i64> {
    use libsql_client::Value as V;
    row.value_map
        .get(name)
        .map(|v| match v {
            &V::Integer { value } => Ok(value),
            _ => Err(anyhow::anyhow!("{} not a 64bit compatible INTEGER", name)),
        })
        .ok_or(anyhow::anyhow!("{} not Found", name))?
}
fn parse_f64_from_row(row: &Row, name: &str) -> anyhow::Result<f64> {
    use libsql_client::Value as V;
    row.value_map
        .get(name)
        .map(|v| match v {
            &V::Float { value } => Ok(value),
            _ => Err(anyhow::anyhow!("{} not a 64bit compatible Float", name)),
        })
        .ok_or(anyhow::anyhow!("{} not Found", name))?
}
fn parse_string_from_row(row: &Row, name: &str) -> anyhow::Result<String> {
    use libsql_client::Value as V;
    row.value_map
        .get(name)
        .map(|v| match v.to_owned() {
            V::Text { value } => Ok(value),
            _ => Err(anyhow::anyhow!("{} not TEXT", name)),
        })
        .ok_or(anyhow::anyhow!("{} not Found", name))?
}
fn parse_pos3_from_row(row: &Row, name: &str) -> anyhow::Result<Pos3> {
    let str = parse_string_from_row(row, name)?;
    let poses = str.split(";");
    let mut poses = poses.map(|v| i32::from_str_radix(v, 10));
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
fn parse_bool_from_row(row: &Row, name: &str) -> anyhow::Result<bool> {
    Ok(match parse_i64_from_row(row, name)? {
        0 => false,
        1 => true,
        _ => anyhow::bail!("number used for bool out of range"),
    })
}
