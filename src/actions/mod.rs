use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

use crate::actions::game_control::{get_movement, GameControl};
use crate::player::Player;
use crate::GameState;

mod game_control;

use crate::graph::grid_transform::*;

pub const FOLLOW_EPSILON: f32 = 5.;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Actions>().add_systems(
            Update,
            set_movement_actions.run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Default, Resource)]
pub struct Actions {
    pub player_movement: Option<GridTransform>,
}

pub fn set_movement_actions(
    mut actions: ResMut<Actions>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player: Query<&Transform, With<Player>>,
) {

    let mut player_movement = ZERO;
    if GameControl::Up.pressed(&keyboard_input) { player_movement = NORTH }
    if GameControl::Down.pressed(&keyboard_input) { player_movement = SOUTH }
    if GameControl::Left.pressed(&keyboard_input) { player_movement = WEST }
    if GameControl::Right.pressed(&keyboard_input) { player_movement = EAST }

    if player_movement != ZERO {
        actions.player_movement = Some(player_movement);
    } else {
        actions.player_movement = None;
    }
}
