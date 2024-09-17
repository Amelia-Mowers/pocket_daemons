use crate::display::scale::SCALE_FACTOR;
use crate::graph::connection::Connection;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

const CONNECTION_THICKNESS: f32 = SCALE_FACTOR * 0.05;

#[derive(Component)]
struct ConnectionDisplay;

#[derive(Bundle)]
struct ConnectionDisplayBundle {
    connection_display: ConnectionDisplay,
    spatial: SpatialBundle,
}

impl Default for ConnectionDisplayBundle {
    fn default() -> Self {
        Self {
            connection_display: ConnectionDisplay,
            spatial: SpatialBundle::default(),
        }
    }
}

pub struct ConnectionDisplayPlugin;

impl Plugin for ConnectionDisplayPlugin { 
    fn build (&self, app: &mut App) {
        app
            .add_systems(Update, (
                spawn_connection_display,
                update_connection_display_transform,
            ));
    }
}

fn spawn_connection_display(
    mut commands: Commands,
    query: Query<(Entity, &Connection), Added<Connection>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let connection_color = Color::BLACK;

    for (entity, connection) in query.iter() {
        commands.entity(entity).insert(ConnectionDisplayBundle {
            spatial: SpatialBundle {
                transform: (*connection).connection_transform(),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Mesh::from(
                    Rectangle::new(1.0, CONNECTION_THICKNESS)
                ))),
                material: materials.add(connection_color),
                transform: Transform::from_xyz(0.0, 0.0, -0.2), 
                ..Default::default()
            });
        });
    }
}

fn update_connection_display_transform(
    mut query: Query<(&Connection, &mut Transform), Changed<Connection>>,
) { 
    for (connection, mut transform) in query.iter_mut() {
        let center = (*connection).connection_transform();
        *transform = center;
    }
}

impl Connection {
    pub fn connection_transform(&self) -> Transform {
        let start_vec = Into::<Transform>::into(self.a).translation;
        let end_vec = Into::<Transform>::into(self.b).translation;
        let middle = (start_vec + end_vec) / 2.0;
        let distance = start_vec.distance(end_vec);
        let angle = (end_vec.y - start_vec.y).atan2(end_vec.x - start_vec.x);

        Transform::from_translation(middle)
            .with_rotation(Quat::from_rotation_z(angle))
            .with_scale(Vec3::new(distance, 1.0, 1.0))
    }
}
