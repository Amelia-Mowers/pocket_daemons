use crate::actions::Actions;
use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;

use crate::graph::grid_transform::*;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(Update, 
                move_player.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, 
                tick_movement_cooldown
            )
            .init_resource::<MovementCooldown>();
    }
}

fn spawn_player(mut commands: Commands, textures: Res<TextureAssets>) {
    commands
        .spawn(SpriteBundle {
            texture: textures.player.clone(),
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
            ..Default::default()
        })
        .insert(Player)
        .insert(ZERO);
}

#[derive(Resource)]
pub struct MovementCooldown(Timer);

pub const PLAYER_SPEED: f32 = 0.25;

impl MovementCooldown {
    pub fn new() -> Self {
        Self(Timer::from_seconds(PLAYER_SPEED, TimerMode::Once))
    }
}

impl Default for MovementCooldown {
    fn default() -> Self {
        Self::new()
    }
}

fn tick_movement_cooldown(
    time: Res<Time>,
    mut timer: ResMut<MovementCooldown>
) {
    timer.0.tick(time.delta());
}

fn move_player(
    actions: Res<Actions>,
    mut move_cooldown: ResMut<MovementCooldown>,
    mut player_query: Query<&mut GridTransform, With<Player>>,
) {
    match actions.player_movement {
        None => return,
        Some(movement) => {
            if move_cooldown.0.finished() {
                for mut player_transform in &mut player_query {
                    *player_transform += movement;
                }
                move_cooldown.0.reset();
            }
        }
    }
}
