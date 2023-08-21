use bevy::prelude::*;

pub struct EventsPlugin;
#[derive(Event)]
pub struct ActiveTurtleChanged(pub i32);
#[derive(Resource)]
pub struct ActiveTurtleRes(pub i32);

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<ActiveTurtleChanged>();
    }
}
