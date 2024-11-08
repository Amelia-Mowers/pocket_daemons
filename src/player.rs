use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;

use crate::graph::grid_transform::*;
use crate::mob::*;
use crate::control::*;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(Update, (
                player_move_control.after(map_inputs_to_control_events),
            ).run_if(in_state(GameState::Playing)));
    }
}

fn spawn_player(mut commands: Commands, textures: Res<TextureAssets>) {
    commands.spawn((
        Player,
        MobBundle {
            texture: textures.player.clone(),
            texture_atlas: TextureAtlas::from(textures.player_layout.clone()),
            ..Default::default()
        },
    ));
}

pub fn player_move_control(
    mut control_events: EventReader<GameControlEvent>,
    mut query: Query<
        &mut MoveTo,
        With<Player>
    >,
) {
    let control = match control_events.read()
    .filter(|e| e.pressed())
    .last() {
        Some(e) => Some(e.control),
        None => None,
    };

    let new_move_to = match control {
        Some(c) => {
            match c {
              GameControl::Up => Some(GridTransform::NORTH),
              GameControl::Down => Some(GridTransform::SOUTH),
              GameControl::Left => Some(GridTransform::WEST),
              GameControl::Right => Some(GridTransform::EAST),
            }
        },
        None => None,
    };
    
    for mut move_to in &mut query {
        *move_to = MoveTo(new_move_to); 
    }
}
