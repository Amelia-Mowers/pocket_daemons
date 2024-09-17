use crate::display::scale::SCALE_FACTOR;
use crate::graph::grid_transform::GridTransform;
use crate::graph::node::Node;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

const NODE_RADIUS: f32 = SCALE_FACTOR * 0.25;
const BORDER_THICKNESS: f32 = SCALE_FACTOR * 0.05;

#[derive(Component)]
struct NodeDisplay;

#[derive(Bundle)]
struct NodeDisplayBundle {
    node_display: NodeDisplay,
    spatial: SpatialBundle,
}

impl Default for NodeDisplayBundle {
    fn default() -> Self {
        Self {
            node_display: NodeDisplay,
            spatial: SpatialBundle::default(),
        }
    }
}

pub struct NodeDisplayPlugin;

impl Plugin for NodeDisplayPlugin { 
    fn build (&self, app: &mut App) {
        app
            .insert_resource(ClearColor(Color::rgb(0.5, 0.5, 0.5)))
            .add_systems(Update, (
                spawn_node_display,
                update_node_display_transform,
            ));
    }

}

fn spawn_node_display(
    mut commands: Commands,
    query: Query<(Entity, &GridTransform), Added<Node>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let fill_color = Color::WHITE;
    let border_color = Color::BLACK;

    for (entity, grid_transform) in query.iter() {
        commands.entity(entity).insert(NodeDisplayBundle {
            spatial: SpatialBundle {
                transform: (*grid_transform).into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Mesh::from(Circle { radius: NODE_RADIUS - BORDER_THICKNESS }))),
                material: materials.add(fill_color),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..Default::default()
            });

            parent.spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Mesh::from(Circle { radius: NODE_RADIUS }))),
                material: materials.add(border_color),
                transform: Transform::from_xyz(0.0, 0.0, -0.1), 
                ..Default::default()
            });
        });
    }
}

fn update_node_display_transform(
    mut query: Query<(&GridTransform, &mut Transform), Changed<GridTransform>>,
) { 
    for (position, mut transform) in query.iter_mut() {
        *transform = (*position).into();
    }
}
