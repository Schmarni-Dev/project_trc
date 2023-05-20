use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::data_types::arc_mutex::ArcMutex;
use anyhow::Ok;
use chrono::{NaiveTime, Utc};
use common::turtle::{Turtle, TurtleIndexType};
use common::world_data::World;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use serde_json::de::from_str;
use serde_json::ser::to_string_pretty;

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
    file_path: PathBuf,
    last_save_time_stamp: NaiveTime,
}

impl DB {
    fn load_db(path: &Path) -> anyhow::Result<DBType> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let db: DBType = from_str(&contents)?;
        Ok(db)
    }
    pub fn new(path: &Path) -> anyhow::Result<DB> {
        let db = Self::load_db(path).unwrap_or(HashMap::new());

        Ok(DB {
            data: db,
            file_path: path.to_owned(),
            last_save_time_stamp: Utc::now().time(),
        })
    }

    fn base(&mut self) -> anyhow::Result<()> {
        info!(
            "time: {}",
            (Utc::now().time() - self.last_save_time_stamp).num_seconds()
        );
        if (Utc::now().time() - self.last_save_time_stamp).num_seconds() >= 120 {
            self.last_save_time_stamp = Utc::now().time();
            self.save()?
        }
        Ok(())
    }

    pub fn push(&mut self, table: DBTables, data: DBDataTypes) -> anyhow::Result<()> {
        self.base()?;
        self.data.insert(table, data);
        Ok(())
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        info!("saving DB");
        let mut f = File::create(&self.file_path)?;
        f.write_all(to_string_pretty(&self.data)?.as_bytes())?;
        Ok(())
    }

    pub fn get_turtle(&mut self, index: TurtleIndexType) -> anyhow::Result<Turtle> {
        self.base()?;
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
        // TODO: FIXME: Fix this Shit why am i using get_turtles here?!
        self.base()?;
        let mut t = self.get_turtles()?;
        t.insert(turtle.index, turtle);
        self.push(DBTables::Turtles, DBDataTypes::Turtles(t))?;
        Ok(())
    }

    pub fn get_turtles(&mut self) -> anyhow::Result<HashMap<i32, Turtle>> {
        self.base()?;
        self.data.entry(DBTables::Turtles).or_insert_with(|| {
            warn!("!!!Turtles Table dosent exist!!!\nreplacing with empty vec");
            DBDataTypes::Turtles(HashMap::new())
        });
        let test = match self.data.get(&DBTables::Turtles) {
            Some(DBDataTypes::Turtles(turtle)) => turtle
                .into_iter()
                .map(|(i, v)| (i.to_owned(), v.clone()))
                .collect(),
            _ => {
                return Err(anyhow::anyhow!("no turltes. lol it's 03:35 help"));
            }
        };
        Ok(test)
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