use bevy::prelude::*;
use bevy_schminput::prelude::*;
use common::{
    client_packets::{C2SPacket, S2CPacket},
    turtle::{get_rotated_orientation, TurnDir},
    Pos3,
};

use crate::{
    turtle::{LocallyControlledTurtle, Turtle, TurtleDirection, TurtlePosition},
    websocket::WebSocketPlugin,
};

pub struct TurtleInputPlugin;

impl Plugin for TurtleInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_actions);
        if app.is_plugin_added::<WebSocketPlugin<C2SPacket, S2CPacket>>() {
        } else {
            app.add_systems(Update, apply_input_local);
            warn!("Websocket Plugin for Turtles not added, using local turtle controller");
        }
    }
}

fn apply_input_local(
    action_query: Query<&BoolActionValue>,
    actions: Res<TurtleMoveActions>,
    mut turtle_query: Query<
        (&mut TurtlePosition, &mut TurtleDirection),
        (With<Turtle>, With<LocallyControlledTurtle>),
    >,
) {
    let forward = action_query.get(actions.move_forward).unwrap().0;
    let back = action_query.get(actions.move_back).unwrap().0;
    let left = action_query.get(actions.turn_left).unwrap().0;
    let right = action_query.get(actions.turn_right).unwrap().0;
    let up = action_query.get(actions.move_up).unwrap().0;
    let down = action_query.get(actions.move_down).unwrap().0;
    for (mut pos, mut dir) in &mut turtle_query {
        if forward {
            **pos += dir.get_forward_vec();
        }
        if back {
            **pos -= dir.get_forward_vec();
        }
        if left {
            **dir = get_rotated_orientation(**dir, TurnDir::Left);
        }
        if right {
            **dir = get_rotated_orientation(**dir, TurnDir::Right);
        }
        if up {
            **pos += Pos3::Y;
        }
        if down {
            **pos += Pos3::NEG_Y;
        }
    }
}

fn setup_actions(mut cmds: Commands) {
    let set = ActionSetHeaderBuilder::new("turtle_movement")
        .with_name("Turtle Movement")
        .build(&mut cmds)
        .id();
    let move_forward = ActionHeaderBuilder::new("move_forward")
        .with_name("Move Forward")
        .with_set(set)
        .build(&mut cmds)
        .insert((
            BoolActionValue::default(),
            KeyboardBindings::default()
                .add_binding(KeyboardBinding::new(KeyCode::KeyW).just_pressed()),
            GamepadBindings::default().add_binding(
                GamepadBindingDevice::Any,
                GamepadBinding::axis(GamepadAxisType::LeftStickY).positive(),
            ),
        ))
        .id();
    let move_back = ActionHeaderBuilder::new("move_back")
        .with_name("Move Backwards")
        .with_set(set)
        .build(&mut cmds)
        .insert((
            BoolActionValue::default(),
            KeyboardBindings::default()
                .add_binding(KeyboardBinding::new(KeyCode::KeyS).just_pressed()),
            GamepadBindings::default().add_binding(
                GamepadBindingDevice::Any,
                GamepadBinding::axis(GamepadAxisType::LeftStickY).negative(),
            ),
        ))
        .id();
    let turn_left = ActionHeaderBuilder::new("turn_left")
        .with_name("Turn Left")
        .with_set(set)
        .build(&mut cmds)
        .insert((
            BoolActionValue::default(),
            KeyboardBindings::default()
                .add_binding(KeyboardBinding::new(KeyCode::KeyA).just_pressed()),
            GamepadBindings::default().add_binding(
                GamepadBindingDevice::Any,
                GamepadBinding::axis(GamepadAxisType::LeftStickX).negative(),
            ),
        ))
        .id();
    let turn_right = ActionHeaderBuilder::new("turn_right")
        .with_name("Turn Right")
        .with_set(set)
        .build(&mut cmds)
        .insert((
            BoolActionValue::default(),
            KeyboardBindings::default()
                .add_binding(KeyboardBinding::new(KeyCode::KeyD).just_pressed()),
            GamepadBindings::default().add_binding(
                GamepadBindingDevice::Any,
                GamepadBinding::axis(GamepadAxisType::LeftStickX).negative(),
            ),
        ))
        .id();
    let move_up = ActionHeaderBuilder::new("move_up")
        .with_name("Move Up")
        .with_set(set)
        .build(&mut cmds)
        .insert((
            BoolActionValue::default(),
            KeyboardBindings::default()
                .add_binding(KeyboardBinding::new(KeyCode::KeyE).just_pressed()),
            // GamepadBindings::default().add_binding(
            //     GamepadBindingDevice::Any,
            //     GamepadBinding::axis(GamepadAxisType::LeftStickY).negative(),
            // ),
        ))
        .id();
    let move_down = ActionHeaderBuilder::new("move_down")
        .with_name("Move Down")
        .with_set(set)
        .build(&mut cmds)
        .insert((
            BoolActionValue::default(),
            KeyboardBindings::default()
                .add_binding(KeyboardBinding::new(KeyCode::KeyQ).just_pressed()),
            // GamepadBindings::default().add_binding(
            //     GamepadBindingDevice::Any,
            //     GamepadBinding::axis(GamepadAxisType::LeftStickY).negative(),
            // ),
        ))
        .id();
    cmds.insert_resource(TurtleMoveActions {
        move_forward,
        move_back,
        turn_left,
        turn_right,
        move_up,
        move_down,
    })
}

#[derive(Resource)]
struct TurtleMoveActions {
    move_forward: Entity,
    move_back: Entity,
    turn_left: Entity,
    turn_right: Entity,
    move_up: Entity,
    move_down: Entity,
}
