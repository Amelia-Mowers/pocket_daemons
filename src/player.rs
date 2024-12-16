use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy::ecs::query::QuerySingleError;

use crate::graph::grid_transform::*;
use crate::mob::*;
use crate::control::*;
use crate::map::*;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                spawn_player.before(player_move_control),
                player_move_control.after(map_inputs_to_control_events),
                player_interact_control.after(map_inputs_to_control_events),
            ).run_if(in_state(GameState::Playing)));
    }
}

pub fn spawn_player(
    mut commands: Commands, 
    mut event: EventReader<PlayerSpawnEvent>,
    mut mob_move_events: EventWriter<MobMoveEvent>,
    textures: Res<TextureAssets>,
    mut query: Query<(
        Entity,
        &mut GridPosition, 
        &mut LastGridPosition, 
        &mut MovementCooldown,
    ), With<Player>>,
    mut map_and_player_loading: ResMut<MapAndPlayerLoading>,
) {
    for event in event.read() {
        let player_entity  = match query.get_single_mut() {
            Ok((entity, mut pos, mut last, mut cooldown)) => {
                *pos = GridPosition(event.location + event.direction);
                *last = LastGridPosition(event.location + event.direction);
                (**cooldown).finish();
                entity
            }
            Err(QuerySingleError::NoEntities(_)) => {
                let new_transform_base:Transform = (event.location + event.direction).into();
                commands.spawn((
                    Player,
                    MobBundle {
                        texture: textures.player.clone(),
                        texture_atlas: TextureAtlas::from(textures.player_layout.clone()),
                        transform: Transform::from_translation(Vec3 {
                             x: new_transform_base.translation.x, 
                             y: new_transform_base.translation.y, 
                             z: 1.0 
                        }),
                        grid_position: GridPosition(event.location + event.direction),
                        ..Default::default()
                    },
                )).id()
            }
            Err(QuerySingleError::MultipleEntities(_)) => {
                panic!("Error: There is more than one player!");
            }
        };
        mob_move_events.send(MobMoveEvent{
            entity: player_entity,
            movement: -event.direction,
        });
        **map_and_player_loading = false;
    } 
}

pub fn player_move_control(
    mut control_events: EventReader<GameControlEvent>,
    mut mob_move_events: EventWriter<MobMoveEvent>,
    query: Query<
        Entity,
        With<Player>
    >,
) {
    match control_events.read()
    .filter(|e| e.pressed())
    .filter(|e| e.is_movement())
    .nth(0) {
        Some(e) => {
            let movement = match e.control {
                GameControl::Up => GridTransform::NORTH,
                GameControl::Down => GridTransform::SOUTH,
                GameControl::Left => GridTransform::WEST,
                GameControl::Right => GridTransform::EAST,
                _ => panic!("Not a Move Control"),
            };
            for player in &query {
                mob_move_events.send(MobMoveEvent{
                    entity: player,
                    movement: movement,
                });
            };
        },
        None => {},
    };
}

pub fn player_interact_control(
    state: Res<State<GameState>>,
    mut control_events: EventReader<GameControlEvent>,
    mut mob_interact_events: EventWriter<MobInteractEvent>,
    query: Query<
        Entity,
        With<Player>
    >,
) {
    match control_events.read()
    .filter(|e| e.just_pressed())
    .filter(|e| e.control == GameControl::Interact)
    .nth(0) {
        Some(_) => {
            if !state.is_changed() {
                for player in &query {
                    mob_interact_events.send(MobInteractEvent{
                        entity: player,
                    });
                };
            }
        },
        None => {},
    };
}

