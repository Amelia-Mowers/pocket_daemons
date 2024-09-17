use bevy::prelude::*;
use crate::graph::grid_transform::GridTransform;

#[derive(Component, Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub struct Connection {
    pub a: GridTransform,
    pub b: GridTransform,
}

impl Connection {
    pub fn new(a: GridTransform, b: GridTransform) -> Self {
        Self {
            a,
            b,
        }
    }

    pub fn crosses(&self, other: &Self) -> bool {
        let dir = self.b - self.a;
        if dir.x != 0 && dir.y != 0 { // Only check diagonal directions
            let intermediate_x = GridTransform::new(self.a.x + dir.x, self.a.y);
            let intermediate_y = GridTransform::new(self.a.x, self.a.y + dir.y);

            let potential_block_a = Connection::new(intermediate_x, intermediate_y);
            let potential_block_b = Connection::new(intermediate_y, intermediate_x);

            *other == potential_block_a ||
            *other == potential_block_b
        } else {
            false       
        }    
    }
}
