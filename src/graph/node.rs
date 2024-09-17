use bevy::prelude::*;
use crate::graph::grid_transform::GridTransform;

#[derive(Component)]
pub struct Node;

#[derive(Bundle)]
pub struct NodeBundle {
    pub node: Node,
    pub position: GridTransform,
}

impl Default for NodeBundle {
    fn default() -> Self {
        Self {
            node: Node,
            position: GridTransform { x: 0, y: 0 },
        }
    }
}


