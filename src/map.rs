use crate::loading::MapAssets;
use crate::GameState;
use bevy::prelude::*;

use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tiled::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnEnter(GameState::Playing), spawn_map)
        .add_systems(Update, mark_map_layer)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .register_tiled_custom_tile::<TileBundle>("TileBundle")
        .register_type::<Terrain>()
        .insert_resource(ClearColor(Color::srgb_u8(24, 48, 48)));
    }
}

#[derive(TiledEnum, Component, Default, Reflect, Debug)]
pub enum Terrain {
    #[default]
    Dirt,
    Grass,
    Tree,
}

pub struct TerrainData {
    pub walkable: bool,
}

impl Terrain {
    pub fn get_terrain_data(&self) -> TerrainData {
        match *self {
            Terrain::Dirt => TerrainData {
                walkable: true,
            },
            Terrain::Grass => TerrainData {
                walkable: true,
            },
            Terrain::Tree => TerrainData {
                walkable: false,
            },
        }
    }
}

#[derive(TiledCustomTile, Bundle, Default, Debug, Reflect)]
struct TileBundle {
    terrain: Terrain,
}

fn spawn_map(
    mut commands: Commands, 
    maps: Res<MapAssets>, 
) {
    commands.spawn(TiledMapBundle {
        tiled_map: maps.test.clone(),
        ..Default::default()
    });
}

#[derive(Component, Default)]
pub struct ProcessedMap;

#[derive(Component, Default)]
pub struct TerrainMap;

fn mark_map_layer(
    mut commands: Commands, 
    mut query: Query<(Entity, &Name), (With<TileStorage>, Without<ProcessedMap>)>,
) {
    for (entity, name) in &mut query {
        commands.entity(entity).insert(ProcessedMap);
        if **name == *"TiledMapTileLayerForTileset(prototype, tiles)" {
            commands.entity(entity).insert(TerrainMap);
        }
    }
}
