use std::collections::HashMap;

use log::info;

use super::server_turtle::ServerTurtle;

pub struct DosentExist;

pub struct TurtleMap(HashMap<i32, ServerTurtle>);

impl TurtleMap {
    pub fn new() -> TurtleMap {
        TurtleMap(HashMap::new())
    }
    pub fn push(&mut self, turtle: ServerTurtle) -> &mut Self {
        info!("Registering Turtle: {}", turtle.index);
        self.0.insert(turtle.index, turtle);
        self
    }
    pub fn get_turtle(&self, id: i32) -> Option<&ServerTurtle> {
        self.0.get(&id)
    }
    pub fn get_turtle_mut(&mut self, id: i32) -> Option<&mut ServerTurtle> {
        info!("HUH?!");
        self.0.keys().for_each(|k| {
            info!("{}", k);
        });
        self.0.get_mut(&id)
    }
}
