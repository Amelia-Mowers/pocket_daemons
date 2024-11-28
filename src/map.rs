use std::collections::HashMap;

use crate::loading::*;
use crate::GameState;
use bevy::prelude::*;

use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::graph::grid_transform::*;
use crate::mob::Mob;
use crate::mob::GridPosition;
use crate::mob::TriggerOnMoveOntoEvent;
use crate::Player;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnEnter(GameState::Playing), init_map)
        .add_systems(Update, (
            index_grid_positions,
            mark_player_spawn,
            change_map,
            map_exits,
        ))
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .register_tiled_object::<PlayerSpawnBundle>("PlayerSpawnBundle")
        .register_tiled_object::<MapExitBundle>("MapExitBundle")
        .register_tiled_custom_tile::<TreeTileBundle>("TreeTileBundle")
        .register_type::<SpawnData>()
        .register_type::<CurrentMap>()
        .register_type::<PreviousMap>()
        .insert_resource(CurrentMap("start".to_string()))
        .insert_resource(PreviousMap("start".to_string()))
        .init_resource::<GridIndex>()
        .add_event::<PlayerSpawnEvent>()
        .add_event::<ChangeMapEvent>()
        .insert_resource(ClearColor(Color::srgb_u8(24, 48, 48)));
    }
}

#[derive(Resource, Reflect, Debug, Default)]
pub struct GridIndex {
    transform_to_entities: HashMap<GridTransform, Vec<Entity>>,
    entity_to_transform: HashMap<Entity, GridTransform>,
}

static EMPTY_VEC: Vec<Entity> = Vec::new();

impl GridIndex {
    /// Updates the index with a new GridTransform for a given Entity
    pub fn update(&mut self, entity: Entity, grid_transform: GridTransform) {
        // Remove entity from old position if it exists
        if let Some(old_transform) = self.entity_to_transform.get(&entity) {
            if let Some(entities) = self.transform_to_entities.get_mut(old_transform) {
                entities.retain(|e| e != &entity);
                // Clean up empty Vec
                if entities.is_empty() {
                    self.transform_to_entities.remove(old_transform);
                }
            }
        }

        // Add entity to new GridTransform
        self.transform_to_entities
            .entry(grid_transform.clone())
            .or_default()
            .push(entity.clone());

        // Update the entity's position
        self.entity_to_transform.insert(entity, grid_transform);
    }

    pub fn get(&self, grid_transform: &GridTransform) -> &Vec<Entity> {
        self.transform_to_entities
            .get(grid_transform)
            .unwrap_or(&EMPTY_VEC)
    }
}

#[derive(TiledClass, Component, Default, Debug, Reflect)]
pub struct IndexGridPosition;

#[derive(Resource, Deref, DerefMut, Reflect, Debug, Default)]
pub struct CurrentMap(String);

#[derive(Resource, Deref, DerefMut, Reflect, Debug, Default)]
pub struct PreviousMap(String);

#[derive(TiledObject, Bundle, Default, Debug, Reflect)]
struct PlayerSpawnBundle {
    #[tiled_rename = "SpawnData"]
    data: SpawnData,
}

#[derive(TiledClass, Component, Default, Debug, Reflect)]
pub struct SpawnData{
    #[tiled_rename = "From"]
    pub from: String,
}


#[derive(TiledClass, Component, Default, Debug, Reflect)]
pub struct TriggerOnMoveOnto;

#[derive(TiledObject, Bundle, Default, Debug, Reflect)]
struct MapExitBundle {
    #[tiled_rename = "ExitData"]
    data: ExitData,
    #[tiled_rename = "IndexGridPostion"]
    index_flag: IndexGridPosition,
    #[tiled_rename = "TriggerOnMoveOnto"]
    block: TriggerOnMoveOnto,
}

#[derive(TiledClass, Component, Default, Debug, Reflect)]
pub struct ExitData{
    #[tiled_rename = "To"]
    pub to: String,
}

#[derive(TiledClass, Component, Default, Debug, Reflect)]
pub struct BlocksWalking;

#[derive(TiledCustomTile, Bundle, Default, Debug, Reflect)]
struct TreeTileBundle {
    #[tiled_rename = "IndexGridPostion"]
    index_flag: IndexGridPosition,
    #[tiled_rename = "BlocksWalking"]
    block: BlocksWalking,
}

#[derive(Event, Deref, DerefMut, Reflect, Debug, Default)]
pub struct ChangeMapEvent(String);

fn init_map(
    mut event: EventWriter<ChangeMapEvent>,
) {
    event.send(ChangeMapEvent("areas/road".to_string()));
}

fn map_exits(
    player_query: Query<Entity, With<Player>>,
    exit_query: Query<&ExitData>,
    mut events: EventReader<TriggerOnMoveOntoEvent>,
    mut change_map_event: EventWriter<ChangeMapEvent>,
) {
    for event in events.read() {
        if player_query.contains(event.moved) {
            if let Ok(exit) = exit_query.get(event.triggered) {
                change_map_event.send(ChangeMapEvent(exit.to.to_string()));
            }
        }
    }
}

fn change_map(
    mut commands: Commands, 
    mut current_map: ResMut<CurrentMap>, 
    mut previous_map: ResMut<PreviousMap>, 
    map_map: Res<MapMap>, 
    mut event: EventReader<ChangeMapEvent>,
    maps: Query<Entity, With<TiledMapMarker>>, 
    mobs: Query<Entity, With<Mob>>, 
) {
    for event in event.read() {
        if let Some(handle) = map_map.get(&event.to_string()) {
            for entity in &maps {
                commands.entity(entity).despawn_recursive();
            }
            for entity in &mobs {
                commands.entity(entity).despawn_recursive();
            }
            *previous_map = PreviousMap(current_map.to_string());
            *current_map = CurrentMap(event.to_string());

            commands.spawn(TiledMapBundle {
                tiled_map: handle.clone(),
                ..Default::default()
            });
        } else {
            warn!("map not in MapMap: {:?}", event);
        }
    }
}

#[derive(Component, Default)]
pub struct ProcessedMap;

#[derive(Component, Default)]
pub struct TerrainMap;

fn index_grid_positions(
    mut commands: Commands, 
    mut query: Query<(Entity, &Transform), With<IndexGridPosition>>,
    mut grid_index: ResMut<GridIndex>,
) {
    for (entity, transform) in &mut query {
        let grid_pos = (*transform).into();
        commands.entity(entity)
        .remove::<IndexGridPosition>()
        .insert(GridPosition(grid_pos));
        grid_index.update(entity, grid_pos);
    }
}

#[derive(Event, Deref, DerefMut, Reflect, Debug, Default)]
pub struct PlayerSpawnEvent(GridTransform);

#[derive(Component)]
pub struct ProcessedPlayerSpawn;

fn mark_player_spawn(
    mut commands: Commands, 
    mut event: EventWriter<PlayerSpawnEvent>,
    query: Query<(Entity, &SpawnData, &Transform), Without<ProcessedPlayerSpawn>>,
    previous_map: Res<PreviousMap>, 
) {
    for (entity, data, transform) in &query {
        info!("Processing player spawn points");
        commands.entity(entity).insert(ProcessedPlayerSpawn);
        if *data.from == **previous_map {
            info!("Spawn point found");
            event.send(PlayerSpawnEvent((*transform).into()));
        }
    }
}
