use std::collections::HashMap;

use crate::loading::*;
use crate::GameState;
use bevy::prelude::*;
use bevy::ecs::query::QuerySingleError;
use bevy::sprite::*;
use crate::Srgba;

use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tiled::prelude::*;

use crate::graph::grid_transform::*;
use crate::mob::Mob;
use crate::mob::GridPosition;
use crate::mob::TriggerOnMoveOntoEvent;
use crate::mob::MovementCooldown;

use crate::mob::TriggerEvent;
use crate::Player;
use crate::RES_WIDTH;
use crate::RES_HEIGHT;
use crate::PIXEL_PERFECT_STATIC_LAYERS;
use crate::text_loading::GameText;
use crate::text_loading::Dialog;
use crate::dialog::CurrentDialog;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
        // .add_systems(OnEnter(GameState::Playing), (
        //     init_map,
        // ))
        .add_systems(Startup, (
            init_transition_effect,
        ))
        .add_systems(Update, (
            index_grid_positions,
            init_sprite,
            mark_player_spawn,
            change_map,
            map_exits,
            transition_effect,
            trigger_dialog,
            update_map_changed.before(change_map),
        ).run_if(in_state(GameState::Playing)))
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .register_tiled_object::<PlayerSpawnBundle>("PlayerSpawnBundle")
        .register_tiled_object::<MapExitBundle>("MapExitBundle")
        .register_tiled_object::<InteractDialogBundle>("InteractDialogBundle")
        .register_tiled_custom_tile::<TreeTileBundle>("TreeTileBundle")
        .register_type::<SpawnData>()
        .register_type::<CurrentMap>()
        .register_type::<CurrentSpawn>()
        .insert_resource(CurrentMap(None))
        .insert_resource(CurrentSpawn(None))
        .init_resource::<GridIndex>()
        .init_resource::<ChangeMapQueue>()
        .init_resource::<MapChangedSinceMove>()
        .init_resource::<MapAndPlayerLoading>()
        .add_event::<PlayerSpawnEvent>()
        .add_event::<TriggerEvent>()
        .insert_resource(ClearColor(Color::srgb_u8(47, 76, 64)));
    }
}

#[derive(Component)]
struct Transition;

fn init_transition_effect (
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(
                RES_WIDTH as f32, 
                RES_HEIGHT as f32,
            ))),
            transform: Transform::from_xyz(
                RES_WIDTH as f32 / 2.0, 
                RES_HEIGHT as f32 / 2.0, 
                2.
            ),
            material: materials.add(Color::srgba_u8(47, 76, 64, 0)),
            ..default()
        },
        PIXEL_PERFECT_STATIC_LAYERS,
        Transition,
    ));
}

fn transition_effect (
    material_handles: Query<&Handle<ColorMaterial>, With<Transition>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player: Query<&MovementCooldown, With<Player>>, 
    change_map_queue: Res<ChangeMapQueue>,
    map_changed: Res<MapChangedSinceMove>, 
    map_and_player_loading: Res<MapAndPlayerLoading>,
) {    
    // Handle player cooldown and errors first.
    let (have_cooldown, fraction) = match player.get_single() {
        Ok(cooldown) => (true, cooldown.fraction()),
        Err(QuerySingleError::NoEntities(_)) => (false, 1.0),
        Err(QuerySingleError::MultipleEntities(_)) => {
            panic!("Error: There is more than one player!");
        }
    };

    let is_queue_empty = change_map_queue.is_empty();
    let is_map_changed = **map_changed;
    let is_loading = **map_and_player_loading;

    // Match on whether the queue is empty and whether the map changed.
    let new_alpha = match (is_queue_empty, is_map_changed) {
        // Queue not empty (map is about to change):
        (false, _) => {
            // If loading, always full alpha.
            // Else if we have a cooldown, use the fraction.
            // Else (no player), full alpha.
            match (is_loading, have_cooldown) {
                (true, _)     => 1.0,
                (false, true) => fraction,
                (false, false)=> 1.0,
            }
        }

        // Queue empty and map changed:
        (true, true) => {
            // If loading, always full alpha.
            // If we have a cooldown, fade out using (1.0 - fraction).
            // Else (no player), full alpha.
            match (is_loading, have_cooldown) {
                (true, _)     => 1.0,
                (false, true) => 1.0 - fraction,
                (false, false)=> 1.0,
            }
        }

        // Queue empty and map not changed: fully transparent.
        (true, false) => 0.0,
    };

    let threshold = 0.8;

    let scaled_alpha = if new_alpha < threshold {
        new_alpha / threshold
    } else {
        1.0
    };
    
    for material_handle in material_handles.iter() {
        if let Some(material) = materials.get_mut(material_handle) {
            if let Color::Srgba(ref mut srgba) = material.color {
                *srgba = Srgba::new(
                    srgba.red,
                    srgba.green,
                    srgba.blue,
                    scaled_alpha,
                );
            }
        }
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
pub struct CurrentMap(Option<Handle<TiledMap>>);

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
    trigger: TriggerOnMoveOnto,
}

#[derive(TiledClass, Component, Default, Debug, Reflect)]
pub struct ExitData{
    #[tiled_rename = "Map"]
    pub map: String,
    #[tiled_rename = "Spawn"]
    pub spawn: String,
}

#[derive(TiledClass, Component, Default, Debug, Reflect)]
pub struct TriggerOnInteract;

#[derive(TiledObject, Bundle, Default, Debug, Reflect)]
struct InteractDialogBundle {
    #[tiled_rename = "Dialog"]
    dialog: DialogReference,
    #[tiled_rename = "IndexGridPostion"]
    index_flag: IndexGridPosition,
    #[tiled_rename = "InitSprite"]
    init_sprite: InitSprite,
    #[tiled_rename = "TriggerOnInteract"]
    trigger: TriggerOnInteract,
    #[tiled_rename = "BlocksWalking"]
    block: BlocksWalking,
}

#[derive(TiledClass, Component, Default, Debug, Reflect)]
pub struct DialogReference {
    #[tiled_rename = "Reference"]
    pub reference: String,
}

#[derive(TiledClass, Component, Default, Debug, Reflect)]
pub struct InitSprite {
    #[tiled_rename = "Reference"]
    pub reference: String,
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

#[derive(Reflect, Debug, Default)]
pub struct ChangeMapEvent {
    pub map: Handle<TiledMap>,
    pub spawn: String,
}

#[derive(Resource, Deref, DerefMut, Reflect, Debug, Default)]
pub struct ChangeMapQueue(Vec<ChangeMapEvent>);

#[derive(Resource, Deref, DerefMut, Reflect, Debug, Default)]
pub struct MapChangedSinceMove(bool);

#[derive(Resource, Deref, DerefMut, Reflect, Debug, Default)]
pub struct MapAndPlayerLoading(bool);

fn update_map_changed(
    mut map_changed: ResMut<MapChangedSinceMove>, 
    map_and_player_loading: Res<MapAndPlayerLoading>,
    player: Query<&MovementCooldown, With<Player>>, 
) {
    match player.get_single() {
        Ok(cooldown) => {
            if cooldown.finished() && !**map_and_player_loading {
                **map_changed = false;
            }
        },
        Err(QuerySingleError::NoEntities(_)) => {
            **map_changed = true;
        },
        Err(QuerySingleError::MultipleEntities(_)) => {
            panic!("Error: There is more than one player!");
        }
    };
}

fn map_exits(
    player_query: Query<Entity, With<Player>>,
    exit_query: Query<&ExitData>,
    mut events: EventReader<TriggerOnMoveOntoEvent>,
    mut change_map_queue: ResMut<ChangeMapQueue>,
    map_assets: Res<MapAssets>, 
) {
    for event in events.read() {
        if player_query.contains(event.moved) {
            if let Ok(exit) = exit_query.get(event.triggered) {
                change_map_queue.push(
                    ChangeMapEvent{
                        map: map_assets.get_field::<Handle<TiledMap>>(&exit.map).unwrap().clone(),
                        spawn: exit.spawn.to_string(),
                    }
                );
            }
        }
    }
}

fn trigger_dialog(
    dialog_query: Query<&DialogReference>,
    mut events: EventReader<TriggerEvent>,
    mut current_dialog: ResMut<CurrentDialog>,
    mut next_state: ResMut<NextState<GameState>>,
    game_text: Res<GameText>,
) {
    for event in events.read() {
        if let Ok(dialog) = dialog_query.get(event.triggered) {
            warn!("Dialog: {}", dialog.reference);
            let next_dialog = game_text.get_field::<Dialog>(&dialog.reference).unwrap();
            *current_dialog = CurrentDialog(Some(next_dialog.clone()));
            next_state.set(GameState::Dialog);
        }
    }
}

fn change_map(
    mut commands: Commands, 
    mut change_map_queue: ResMut<ChangeMapQueue>,
    mut current_spawn: ResMut<CurrentSpawn>, 
    mut current_map: ResMut<CurrentMap>, 
    mut map_changed: ResMut<MapChangedSinceMove>, 
    maps: Query<Entity, With<TiledMapMarker>>, 
    mobs: Query<Entity, (With<Mob>, Without<Player>)>, 
    player: Query<&MovementCooldown, With<Player>>, 
    mut map_and_player_loading: ResMut<MapAndPlayerLoading>,
) {
    if !change_map_queue.is_empty() {
        let cooldown_finished = match player.get_single() {
            Ok(cooldown) => cooldown.finished(),
            Err(QuerySingleError::NoEntities(_)) => true,
            Err(QuerySingleError::MultipleEntities(_)) => {
                panic!("Error: There is more than one player!");
            }
        };
        if cooldown_finished {
            match change_map_queue.last() {
                Some(event) => {
                    for entity in &maps {
                        commands.entity(entity).despawn_recursive();
                    }
                    for entity in &mobs {
                        commands.entity(entity).despawn_recursive();
                    }
                    *current_map = CurrentMap(Some(event.map.clone()));
                    *current_spawn = CurrentSpawn(Some(event.spawn.to_string()));

                    commands.spawn(TiledMapBundle {
                        tiled_map: event.map.clone(),
                        ..Default::default()
                    });
                    **map_changed = true;
                    **map_and_player_loading = true;
                },
                None => {},
            };
            change_map_queue.clear();
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

fn init_sprite(
    mut commands: Commands, 
    mut query: Query<(Entity, &InitSprite, &mut Transform)>,
    textures: Res<TextureAssets>,
) {
    for (entity, init_sprite, mut transform) in &mut query {
        commands.entity(entity)
        .remove::<InitSprite>()
        .insert((
            textures.get_field::<Handle<Image>>(&init_sprite.reference).unwrap().clone(),   
            Sprite {
                anchor: Anchor::Custom(Vec2::new(-0.5, -(14.0/16.0))),
                ..Default::default()
            },
        ));
        *transform = Transform::from_translation(Vec3 {
             x: transform.translation.x, 
             y: transform.translation.y, 
             z: 0.5 
        });
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
                    _ => { panic!("invalid spawn direction") }
                },
            });
        }
    }
}
