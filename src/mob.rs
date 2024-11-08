use crate::GameState;
use bevy::prelude::*;
use bevy::sprite::*;

use crate::graph::grid_transform::*;
use crate::player::*;

use crate::map::*;
use bevy_ecs_tilemap::prelude::*;

#[derive(Component, Default)]
pub struct Mob;

#[derive(Component, Deref, DerefMut, Reflect, Debug)]
pub struct GridDirection(GridTransform);

impl Default for GridDirection {
    fn default() -> Self {
        GridDirection(GridTransform::SOUTH)
    }
}

#[derive(Component, Deref, DerefMut, Reflect, Debug, Default)]
pub struct GridPosition(GridTransform);

#[derive(Component, Deref, DerefMut, Reflect, Debug, Default)]
pub struct LastGridPosition(GridTransform);

#[derive(Component, Deref, DerefMut, Reflect, Debug, Default)]
pub struct MoveTo(pub Option<GridTransform>);

#[derive(Component, Deref, DerefMut, Reflect, Debug, Default)]
pub struct AnimationIndex(usize);

#[derive(Component, Deref, DerefMut, Reflect, Debug)]
pub struct AnimationTimer(Timer);

impl Default for AnimationTimer {
    fn default() -> Self {
        AnimationTimer(Timer::from_seconds(
            0.1, 
            TimerMode::Repeating
        ))
    }
}

#[derive(Component, Deref, DerefMut, Reflect, Debug)]
pub struct MovementCooldown(Timer);

impl Default for MovementCooldown {
    fn default() -> Self {
        MovementCooldown(Timer::from_seconds(
            0.4, 
            TimerMode::Once
        ))
    }
}

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            move_mob.after(player_move_control),
        ).run_if(in_state(GameState::Playing)))
        .register_type::<GridDirection>()
        .register_type::<GridPosition>()
        .register_type::<LastGridPosition>()
        .register_type::<MoveTo>()
        .register_type::<MovementCooldown>()
        .register_type::<GridTransform>();
    }
}

#[derive(Bundle)]
pub struct MobBundle {
    pub mob: Mob,
    pub grid_position: GridPosition,
    pub last_grid_position: LastGridPosition,
    pub move_to: MoveTo,
    pub grid_direction: GridDirection,
    pub movement_cooldown: MovementCooldown,
    pub texture_atlas: TextureAtlas,
    pub animation_index: AnimationIndex,
    pub animation_timer: AnimationTimer,

    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub texture: Handle<Image>,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl Default for MobBundle {
    fn default() -> Self {
        Self {
            mob: Default::default(),
            grid_position: Default::default(),
            last_grid_position: Default::default(),
            move_to: Default::default(),
            grid_direction: Default::default(),
            movement_cooldown: Default::default(),
            texture_atlas: Default::default(),
            animation_index: Default::default(),
            animation_timer: Default::default(),

            sprite: Sprite {
                anchor: Anchor::BottomLeft,
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3 { x: 0., y: 0., z: 1. }),
            global_transform: Default::default(),
            texture: Default::default(),
            visibility: Default::default(),
            inherited_visibility: Default::default(),
            view_visibility: Default::default(),
        }
    }
}

fn move_mob(
    time: Res<Time>,
    mut query: Query<(
        &mut GridPosition, 
        &mut LastGridPosition, 
        &mut GridDirection,
        &mut MoveTo,
        &mut MovementCooldown,
    ), With<Mob>>,
    map_query: Query<&TileStorage, With<TerrainMap>>,
    tile_query: Query<&Terrain>,
) {
    // Retrieve map and return early if unavailable
    let map_storage = match map_query.get_single() {
        Ok(storage) => storage,
        Err(_) => return,
    };

    for (mut pos, mut last_pos, mut dir, mut move_to, mut cooldown) in &mut query {
        cooldown.tick(time.delta());

        // Proceed only if a move command exists and cooldown is complete
        if let Some(direction) = move_to.0 {
            if !cooldown.finished() {
                continue;
            }

            **dir = direction;
            let new_pos = **pos + direction;

            // Try converting new position to a tile position
            if let Some(tile_pos) = new_pos.to_tile_pos() {
                // Check if a tile exists at the new position
                if let Some(tile) = map_storage.checked_get(&tile_pos) {
                    // Confirm that the terrain is walkable
                    if let Ok(terrain) = tile_query.get(tile) {
                        if terrain.get_terrain_data().walkable {
                            **last_pos = pos.0;
                            **pos = new_pos;
                            cooldown.reset();
                        }
                    }
                }
            }

            // Clear the move command after processing
            *move_to = MoveTo(None);
        }
    }
}
