use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
// use bevy_kira_audio::AudioSource;

use bevy_ecs_tiled::prelude::*;
use std::collections::HashMap;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<MapMap>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu)
                .load_collection::<AudioAssets>()
                .load_collection::<TextureAssets>()
                // .load_collection::<MapAssets>()
        )
        .add_systems(OnEnter(GameState::Loading),
            init_map_map,
        );
    }
}

const MAPLIST: &[&str] = &[
    "rules_test",
    "areas/road",
    "areas/clearing",
];

#[derive(Default, Resource, Deref)]
pub struct MapMap(HashMap<String, Handle<TiledMap>>);

pub fn init_map_map(
    mut map_map: ResMut<MapMap>,
    asset_server: Res<AssetServer>,
) {
    // Initialize the map with each map handle loaded by the asset server
    *map_map = MapMap(
        MAPLIST.iter()
            .map(|&map_name| {
                let map_handle: Handle<TiledMap> = asset_server.load(
                    &format!("maps/{}_emb.tmx", map_name)
                );
                (map_name.to_string(), map_handle)
            })
            .collect()
    );
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    // #[asset(path = "audio/flying.ogg")]
    // pub flying: Handle<AudioSource>,
}

// #[derive(AssetCollection, Resource)]
// pub struct MapAssets{
//     #[asset(path = "maps/rules_test_emb.tmx")]
//     pub test: Handle<TiledMap>,
// }

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    
    pub bevy: Handle<Image>,
    
    #[asset(texture_atlas_layout(
        tile_size_x = 16, 
        tile_size_y = 16, 
        columns = 4, 
        rows = 4
    ))]
    pub player_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "sprites/player.png")]
    pub player: Handle<Image>,

    #[asset(path = "textures/github.png")]
    pub github: Handle<Image>,
}
