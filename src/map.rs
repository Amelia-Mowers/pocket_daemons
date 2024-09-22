use crate::actions::Actions;
use crate::loading::TextureAssets;
use crate::loading::MapAssets;
use crate::GameState;
use bevy::prelude::*;

use crate::graph::grid_transform::*;

use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_map)
        .add_plugins(TilemapPlugin)
        .add_plugins(TiledMapPlugin)
        .insert_resource(ClearColor(Color::rgb_u8(24, 48, 48)));
    }
}

fn spawn_map(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    maps: Res<MapAssets>
) {
    // commands
    //     .spawn((
    //         SpriteBundle {
    //             texture: textures.map.clone(),
    //             transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
    //             ..Default::default()
    //         },
    //         Map,
    //     ));

    // let map_handle: Handle<TiledMap> = asset_server.load("sprites/rules_test.tmx");

    // Spawn the map with default options
    commands.spawn(TiledMapBundle {
        tiled_map: maps.map.clone(),
        // transform: Transform::from_translation(Vec3::new(0., 0., -1.0)),
        ..Default::default()
    });
}
