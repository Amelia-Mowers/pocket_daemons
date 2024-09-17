use bevy::prelude::*;

use crate::graph::grid_transform::GridTransform;
use crate::graph::node::Node;
use crate::display::scale::SCALE_FACTOR;


#[derive(Default, Resource)]
struct BoundsTracker {
    min_x: i16,
    max_x: i16,
    min_y: i16,
    max_y: i16,
}

impl BoundsTracker {
    fn update(&mut self, x: i16, y: i16) {
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);
        self.min_y = self.min_y.min(y);
        self.max_y = self.max_y.max(y);
    }
    
    fn center(&self) -> Vec3 {
        let mid_x = (self.min_x as f32 + self.max_x as f32) * SCALE_FACTOR / 2.0;
        let mid_y = (self.min_y as f32 + self.max_y as f32) * SCALE_FACTOR / 2.0;
        Vec3::new(mid_x, mid_y, 0.0)
    }
}

pub struct NodeBoundsDisplayPlugin;

impl Plugin for NodeBoundsDisplayPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(BoundsTracker::default())
            .add_systems(Update, (
                update_bounds,
                // center_camera,
            ));
    }
}

fn update_bounds(
    query: Query<&GridTransform, (With<Node>, Changed<GridTransform>)>,
    mut bounds: ResMut<BoundsTracker>,
) {
    for grid_transform in query.iter() {
        let x = grid_transform.x;
        let y = grid_transform.y;
        bounds.update(x, y);
    }
}

// fn center_camera(
//     mut query: Query<(&Camera, &mut Transform)>,
//     bounds: Res<BoundsTracker>,
// ) {
//     for (_, mut Transform) in &mut query {
//         Transform.translation = bounds.center();
//     }
// }
