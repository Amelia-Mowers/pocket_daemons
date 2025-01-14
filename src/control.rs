use std::collections::HashMap;

use bevy::prelude::*;
use bevy::prelude::{
    ButtonInput, 
    KeyCode, 
    Res,
};

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<InputMap>()
        .add_event::<GameControlEvent>()
        .add_systems(
            Startup,
            init_input_map,
        )
        .add_systems(
            Update,
            map_inputs_to_control_events,
        );
    }
}

#[derive(Hash, PartialEq, Eq)]
pub enum Input {
    Keyboard(KeyCode),
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameControl {
    Up,
    Down,
    Left,
    Right,
    Interact,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ControlStatus {
    Pressed,
    JustPressed,
    // JustReleased,
}

#[derive(Event, Debug, PartialEq)]
pub struct GameControlEvent {
    pub control: GameControl,
    pub status: ControlStatus,
}

impl GameControlEvent {
    pub fn pressed(&self) -> bool {
        self.status == ControlStatus::Pressed
    }
    pub fn just_pressed(&self) -> bool {
        self.status == ControlStatus::JustPressed
    }
    pub fn is_movement(&self) -> bool {
        match self.control {
            GameControl::Up => true,
            GameControl::Down => true,
            GameControl::Left => true,
            GameControl::Right => true,
            _ => false,
        }
    }
}

#[derive(Default, Resource, Deref)]
// #[deref(forward)]
pub struct InputMap(HashMap<Input, GameControl>);

pub fn init_input_map(
    mut input_map: ResMut<InputMap>,
) {
    *input_map = InputMap(HashMap::from([
        (Input::Keyboard(KeyCode::KeyW), GameControl::Up),
        (Input::Keyboard(KeyCode::KeyS), GameControl::Down),
        (Input::Keyboard(KeyCode::KeyA), GameControl::Left),
        (Input::Keyboard(KeyCode::KeyD), GameControl::Right),
        (Input::Keyboard(KeyCode::ArrowUp), GameControl::Up),
        (Input::Keyboard(KeyCode::ArrowDown), GameControl::Down),
        (Input::Keyboard(KeyCode::ArrowLeft), GameControl::Left),
        (Input::Keyboard(KeyCode::ArrowRight), GameControl::Right),
        (Input::Keyboard(KeyCode::Space), GameControl::Interact),
        (Input::Keyboard(KeyCode::Enter), GameControl::Interact),
    ]));
}

pub fn map_inputs_to_control_events(
    mut control: EventWriter<GameControlEvent>,
    input_map: Res<InputMap>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for key in keys.get_pressed() {
        match input_map.get(&Input::Keyboard(*key)) {
            Some(c) => {
                control.send(GameControlEvent{
                   control: *c,
                   status: ControlStatus::Pressed,
                });
            },
            None => {},
        }
    }
    for key in keys.get_just_pressed() {
        match input_map.get(&Input::Keyboard(*key)) {
            Some(c) => {
                control.send(GameControlEvent{
                   control: *c,
                   status: ControlStatus::JustPressed,
                });
            },
            None => {},
        }
    }
}
