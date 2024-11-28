use bevy::prelude::*;
use std::ops::{Add, AddAssign, Sub, Neg};
use std::convert::TryFrom;
use bevy_ecs_tilemap::prelude::*;

// use crate::graph::connection::Connection;

pub const SCALE_FACTOR: f32 = 16.0;

#[derive(Component, Debug, PartialEq, Eq, Hash, Default, Clone, Copy, Reflect)]
pub struct GridTransform {
    pub x: i16,
    pub y: i16,
}

impl GridTransform {
    pub const fn new(x: i16, y: i16) -> Self {
        GridTransform { x, y }
    }

    pub fn neighbors(&self) -> Vec<Self> {
        let mut neighbors = Vec::new();
        for dir in Self::CARDINALS {
            neighbors.push(*self + dir);
        } 
        neighbors
    }
    
    pub fn cardinal_index(&self) -> usize {
        // Logical statements that evaluate to 1 if true, 0 if false
        let is_north = ((self.x == 0) && (self.y == 1)) as usize;
        let is_east = ((self.x == 1) && (self.y == 0)) as usize;
        let is_south = ((self.x == 0) && (self.y == -1)) as usize;
        let is_west = ((self.x == -1) && (self.y == 0)) as usize;

        // Multiply each boolean by its index and sum them up
        let index = is_north * 0 + is_east * 1 + is_south * 2 + is_west * 3;

        index
    }
    
    // pub fn neighbors_ordinals(&self, connections: Vec<Connection>) -> Vec<Self> {
    //     let mut neighbors = Vec::new();

    //     for &dir in &ORDINALS {
    //         let new_conn = Connection::new(*self, *self + dir);

    //         // Check if the diagonal is blocked by any connections
    //         let blocked = connections.iter().any(|conn| 
    //             new_conn.crosses(conn) 
    //         );

    //         if !blocked {
    //             neighbors.push(*self + dir);
    //         }
    //     }

    //     neighbors
    // }

    pub const ZERO: GridTransform = GridTransform { x: 0, y: 0 };

    pub const NORTH: GridTransform = GridTransform { x: 0, y: 1 };
    pub const NORTH_EAST: GridTransform = GridTransform { x: 1, y: 1 };
    pub const EAST: GridTransform = GridTransform { x: 1, y: 0 };
    pub const SOUTH_EAST: GridTransform = GridTransform { x: 1, y: -1 };
    pub const SOUTH: GridTransform = GridTransform { x: 0, y: -1 };
    pub const SOUTH_WEST: GridTransform = GridTransform { x: -1, y: -1 };
    pub const WEST: GridTransform = GridTransform { x: -1, y: 0 };
    pub const NORTH_WEST: GridTransform = GridTransform { x: -1, y: 1 };

    pub const CARDINALS: [GridTransform; 4] = [
        Self::NORTH,
        Self::EAST,
        Self::SOUTH,
        Self::WEST,
    ];

    pub const ORDINALS: [GridTransform; 8] = [
        Self::NORTH,
        Self::NORTH_EAST,
        Self::EAST,
        Self::SOUTH_EAST,
        Self::SOUTH,
        Self::SOUTH_WEST,
        Self::WEST,
        Self::NORTH_WEST,
    ];

}

impl TryFrom<GridTransform> for TilePos {
    type Error = &'static str;

    fn try_from(value: GridTransform) -> Result<Self, Self::Error> {
        if value.x >= 0 && value.y >= 0 {
            Ok(TilePos {
                x: value.x as u32,
                y: value.y as u32,
            })
        } else {
            Err("Negative coordinates are not valid for TilePos")
        }
    }
}
    
impl From<Transform> for GridTransform {
    fn from(value: Transform) -> GridTransform {
        GridTransform {
            x: (value.translation.x / SCALE_FACTOR) as i16,
            y: (value.translation.y / SCALE_FACTOR) as i16,
        }
    }
}
    
impl Add for GridTransform {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        GridTransform {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for GridTransform {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for GridTransform {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        GridTransform {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Neg for GridTransform {
    type Output = Self;

    fn neg(self) -> Self::Output {
        GridTransform {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl PartialOrd for GridTransform {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GridTransform {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.x.cmp(&other.x) {
            std::cmp::Ordering::Equal => self.y.cmp(&other.y),
            other => other,
        }
    }
}
