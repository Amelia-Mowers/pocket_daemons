use crate::loading::TextureAssets;
// use crate::loading::MapAssets;
use crate::GameState;
// use crate::helpers::tiled::*;
use bevy::prelude::*;
use bevy::sprite::*;

use crate::graph::grid_transform::*;

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
        .insert_resource(ClearColor(Color::rgb_u8(24, 48, 48)));
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

#[derive(Component)]
pub struct Map;

fn spawn_map(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
) {
    let map_handle: Handle<TiledMap> = asset_server.load("maps/rules_test_emb.tmx");

    commands.spawn(TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

#[derive(Component, Default)]
pub struct ProcessedMap;

#[derive(Component, Default)]
pub struct TerrainMap;

fn mark_map_layer(
    mut commands: Commands, 
    mut query: Query<(Entity, &Name, &TileStorage), Without<ProcessedMap>>,
) {
    for (entity, name, storage) in &mut query {
        commands.entity(entity).insert(ProcessedMap);
        if **name == *"TiledMapTileLayerForTileset(prototype, tiles)" {
            commands.entity(entity).insert(TerrainMap);
        }
    }
}
