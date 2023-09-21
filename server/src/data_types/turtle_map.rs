use std::collections::HashMap;

use common::turtle::Turtle;
use log::info;

use super::server_turtle::ServerTurtle;

pub struct DosentExist;

pub struct TurtleMap(HashMap<i32, ServerTurtle>);

impl TurtleMap {
    pub fn new() -> TurtleMap {
        TurtleMap(HashMap::new())
    }
    pub fn push(&mut self, turtle: ServerTurtle) -> &mut Self {
        info!("Registering Turtle: {}, {}",&turtle.world, &turtle.index);
        self.0.insert(turtle.get_instance_id(), turtle);
        self
    }
    pub fn get_common_turtles(&self) -> Vec<Turtle> {
        self.0.values().map(|st| Turtle::clone(&st)).collect()
    }
    pub fn get_turtle(&self, id: i32) -> Option<&ServerTurtle> {
        self.0.get(&id)
    }
    pub fn get_turtle_mut(&mut self, id: i32) -> Option<&mut ServerTurtle> {
        self.0.get_mut(&id)
    }
    pub fn drop_turtle(&mut self, id: &i32) {
        if let Some(t) = self.0.remove(id) {
            t.kill();
        }
    }
}
