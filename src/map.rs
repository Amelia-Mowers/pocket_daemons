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
        .register_type::<CurrentSpawn>()
        .insert_resource(CurrentMap(None))
        .insert_resource(CurrentSpawn(None))
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
pub struct CurrentMap(Option<String>);

#[derive(Resource, Deref, DerefMut, Reflect, Debug, Default)]
pub struct CurrentSpawn(Option<String>);

#[derive(TiledObject, Bundle, Default, Debug, Reflect)]
struct PlayerSpawnBundle {
    #[tiled_rename = "SpawnData"]
    data: SpawnData,
}

#[derive(TiledClass, Component, Default, Debug, Reflect)]
pub struct SpawnData{
    #[tiled_rename = "Name"]
    pub name: String,
    #[tiled_rename = "FromDirection"]
    pub from_direction: String,
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
    #[tiled_rename = "Map"]
    pub map: String,
    #[tiled_rename = "Spawn"]
    pub spawn: String,
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

#[derive(Event, Reflect, Debug, Default)]
pub struct ChangeMapEvent {
    pub map: String,
    pub spawn: String,
}

fn init_map(
    mut event: EventWriter<ChangeMapEvent>,
) {
    event.send(ChangeMapEvent{
        map: "areas/road".to_string(), 
        spawn: "start".to_string()
    });
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
                change_map_event.send(
                    ChangeMapEvent{
                        map: exit.map.to_string(),
                        spawn: exit.spawn.to_string(),
                    }
                );
            }
        }
    }
}

fn change_map(
    mut commands: Commands, 
    mut current_map: ResMut<CurrentMap>, 
    mut current_spawn: ResMut<CurrentSpawn>, 
    map_map: Res<MapMap>, 
    mut event: EventReader<ChangeMapEvent>,
    maps: Query<Entity, With<TiledMapMarker>>, 
    mobs: Query<Entity, (With<Mob>, Without<Player>)>, 
) {
    for event in event.read() {
        if let Some(handle) = map_map.get(&event.map.to_string()) {
            for entity in &maps {
                commands.entity(entity).despawn_recursive();
            }
            for entity in &mobs {
                commands.entity(entity).despawn_recursive();
            }
            *current_map = CurrentMap(Some(event.map.to_string()));
            *current_spawn = CurrentSpawn(Some(event.spawn.to_string()));

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

#[derive(Event, Reflect, Debug, Default)]
pub struct PlayerSpawnEvent {
    pub location: GridTransform,
    pub direction: GridTransform,
}

#[derive(Component)]
pub struct ProcessedPlayerSpawn;

fn mark_player_spawn(
    mut commands: Commands, 
    mut event: EventWriter<PlayerSpawnEvent>,
    query: Query<(Entity, &SpawnData, &Transform), Without<ProcessedPlayerSpawn>>,
    current_spawn: Res<CurrentSpawn>,
) {
    for (entity, data, transform) in &query {
        info!("Processing player spawn points");
        commands.entity(entity).insert(ProcessedPlayerSpawn);
        if Some((*data.name).to_string()) == **current_spawn {
            info!("Spawn point found");
            event.send(PlayerSpawnEvent{
                location: (*transform).into(),
                direction: match &data.from_direction {
                    val if *val == "north".to_string() => GridTransform::NORTH,
                    val if *val == "east".to_string()  => GridTransform::EAST,
                    val if *val == "south".to_string()  => GridTransform::SOUTH,
                    val if *val == "west".to_string()  => GridTransform::WEST,
                    val if *val == "center".to_string()  => GridTransform::ZERO,
                    _ => {panic!("invalid spawn condition");}
                },
            });
        }
    }
}
