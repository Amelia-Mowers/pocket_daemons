use crate::actions::Actions;
use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;

use crate::graph::grid_transform::*;

pub struct MapPlugin;

#[derive(Component)]
pub struct Map;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_map)
        .insert_resource(ClearColor(Color::rgb_u8(24, 48, 48)));
    }
}

fn spawn_map(mut commands: Commands, textures: Res<TextureAssets>) {
    commands
        .spawn((
            SpriteBundle {
                texture: textures.map.clone(),
                transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                ..Default::default()
            },
            Map,
        ));
}
