use anyhow::Ok;
use common::turtle::{Orientation, Turtle, TurtleIndexType};
use common::world_data::World;
use common::Pos3;
use libsql_client::{args, Client, Row, Statement};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use serde_json::de::from_str;
use serde_json::ser::to_string_pretty;
use std::cell::OnceCell;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

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
    data: DBType,
    pub client: Client,
    file_path: PathBuf,
}

fn get_turtle_instance_from_db(row: Row) -> anyhow::Result<Turtle> {
    let index: i32 = parse_i64_from_row(&row, "index")?.try_into()?;
    let name = parse_string_from_row(&row, "name")?;
    let inv = serde_json::from_str(&parse_string_from_row(&row, "inventory")?)?;
    let pos = parse_pos3_from_row(&row, "position")?;
    let orient = Orientation::from_str(&parse_string_from_row(&row, "orientation")?)
        .map_err(|e| anyhow::anyhow!(e))?;
    let fuel = parse_f64_from_row(&row, "fuel")? as f32;
    let max_fuel: i32 = parse_i64_from_row(&row, "max_fuel")?.try_into()?;
    Ok(Turtle::new(
        index, name, inv, pos, orient, fuel, max_fuel, false,
    ))
}

fn pos_to_db_pos(pos: &Pos3) -> Arc<str> {
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
    let x = pos.x.abs() as u32 & get_chunk_key_mask(0);
    let y = pos.y.abs() as u32 & get_chunk_key_mask(1);
    let z = pos.z.abs() as u32 & get_chunk_key_mask(2);
    return x | y | z;
}

impl DB {
    async fn setup(dbd: DBType, filepath: &Path) -> anyhow::Result<DB> {
        let db = libsql_client::Client::from_env().await?;
        db.execute(
            "
        CREATE TABLE IF NOT EXISTS turtles (
            id INTEGER NOT NULL PRIMARY KEY,
            name TEXT NOT NULL,
            inventory TEXT NOT NULL,
            posistion TEXT NOT NULL,
            orientation TEXT NOT NULL,
            fuel FLOAT NOT NULL,
            max_fuel INTEGER NOT NULL,
            world TEXT NOT NULL,
            FOREIGN KEY (world)
                REFERENCES worlds (name)
        );",
        )
        .await?;
        info!("turtles");
        db.execute(
            "
        CREATE TABLE IF NOT EXISTS worlds (
            name TEXT NOT NULL PRIMARY KEY

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
            FOREIGN KEY (world)
                REFERENCES worlds (name)
        );",
        )
        .await?;
        Ok(DB {
            data: dbd,
            file_path: filepath.to_owned(),
            client: db,
        })
    }
    fn load_db(path: &Path) -> anyhow::Result<DBType> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let db: DBType = from_str(&contents)?;
        Ok(db)
    }
    pub async fn new(path: &Path) -> anyhow::Result<DB> {
        let db = Self::load_db(path).unwrap_or(HashMap::new());

        DB::setup(db, path).await
    }
    pub async fn transfer(&mut self) -> anyhow::Result<()> {
        let world = self.get_world();
        let turtles = self.get_turtles()?;
        self.client
            .execute(Statement::with_args(
                "INSERT INTO worlds VALUES (?)",
                args!("test_world_1"),
            ))
            .await?;
        let t = self.client.transaction().await?;
        for (cp, c) in world.get_chunks().to_owned().iter() {
            for (_, b) in c.all_blocks().iter() {
                t.execute(Statement::with_args(
                    "INSERT INTO blocks VALUES (?,?,?,?,?);",
                    args!(
                        pos_to_key(cp),
                        &b.id,
                        "test_world_1",
                        *pos_to_db_pos(&b.pos),
                        match b.is_air {
                            true => 1,
                            false => 0,
                        }
                    ),
                ))
                .await?;
            }
        }
        info!("blocks done");
        t.commit().await?;

        let t = self.client.transaction().await?;
        for tu in turtles {
            t.execute(Statement::with_args(
                "INSERT INTO turtles VALUES (?,?,?,?,?,?,?,?);",
                args!(
                    tu.index,
                    tu.name,
                    serde_json::to_string(&tu.inventory)?,
                    *pos_to_db_pos(&tu.position),
                    tu.orientation.to_string(),
                    tu.fuel,
                    tu.max_fuel,
                    "test_world_1",
                ),
            ))
            .await?;
        }
        t.commit().await?;
        Ok(())
    }

    pub fn get_world(&self) -> World {
        let world = match self
            .data
            .get(&DBTables::World)
            .unwrap_or(&DBDataTypes::World(World::new()))
        {
            DBDataTypes::Turtles(_) => panic!("DB Fuckup"),
            DBDataTypes::World(world) => world.clone(),
        };
        world
    }

    pub fn set_world(&mut self, world: World) {
        self.push(DBTables::World, DBDataTypes::World(world))
    }

    // fn base(&mut self) -> anyhow::Result<()> {
    //     info!(
    //         "time: {}",
    //         (Utc::now().time() - self.last_save_time_stamp).num_seconds()
    //     );
    //     if (Utc::now().time() - self.last_save_time_stamp).num_seconds() >= 120 {
    //         self.last_save_time_stamp = Utc::now().time();
    //         self.save()?
    //     }
    //     Ok(())
    // }

    pub fn push(&mut self, table: DBTables, data: DBDataTypes) {
        self.data.insert(table, data);
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        info!("saving DB");
        let mut f = File::create(&self.file_path)?;
        f.write_all(to_string_pretty(&self.data)?.as_bytes())?;
        info!("saved DB");
        Ok(())
    }

    pub fn get_turtle(&mut self, index: TurtleIndexType) -> anyhow::Result<Turtle> {
        match self.data.get(&DBTables::Turtles).iter().find_map(|t| {
            if let DBDataTypes::Turtles(turlte) = t {
                turlte.get(&index)
            } else {
                None
            }
        }) {
            None => Err(anyhow::anyhow!("No Turtle Found")),
            Some(t) => Ok(t.clone()),
        }
    }

    pub fn push_turtle(&mut self, turtle: Turtle) -> anyhow::Result<()> {
        // TODO: FIXME: Fix this Shit why tf am i using get_turtles here?!

        let mut t = self.get_turtle_map()?;
        t.insert(turtle.index, turtle);
        self.push(DBTables::Turtles, DBDataTypes::Turtles(t));
        Ok(())
    }

    fn get_turtle_map(&mut self) -> anyhow::Result<HashMap<i32, Turtle>> {
        self.data.entry(DBTables::Turtles).or_insert_with(|| {
            warn!("!!!Turtles Table dosent exist!!!\nreplacing with empty vec");
            DBDataTypes::Turtles(HashMap::new())
        });
        match self.data.get(&DBTables::Turtles) {
            Some(DBDataTypes::Turtles(turtle)) => Ok(turtle
                .into_iter()
                .map(|(i, v)| (i.to_owned(), v.clone()))
                .collect()),
            _ => Err(anyhow::anyhow!("no turltes. lol it's 03:35 help")),
        }
    }

    pub fn get_turtles(&mut self) -> anyhow::Result<Vec<Turtle>> {
        Ok(self
            .get_turtle_map()?
            .values()
            .map(|t| t.to_owned())
            .collect())
        // todo!();
    }

    pub fn contains_turtle(&self, index: TurtleIndexType) -> bool {
        self.data.get(&DBTables::Turtles).iter().any(|t| {
            if let DBDataTypes::Turtles(turlte) = t {
                return turlte.contains_key(&index);
            };
            false
        })
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
