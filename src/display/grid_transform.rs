use bevy::prelude::*;
use crate::graph::grid_transform::GridTransform;
use crate::display::scale::SCALE_FACTOR;

impl From<GridTransform> for Transform {
    fn from(position: GridTransform) -> Self {
        Transform::from_xyz(
            position.x as f32 * SCALE_FACTOR, 
            position.y as f32 * SCALE_FACTOR, 
            0.0
        )
    }
}
