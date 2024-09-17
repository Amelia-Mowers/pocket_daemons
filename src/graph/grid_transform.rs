use bevy::prelude::*;
use std::ops::{Add, AddAssign, Sub, Neg};

// use crate::graph::connection::Connection;

#[derive(Component, Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
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
        for dir in CARDINALS {
            neighbors.push(*self + dir);
        } 
        neighbors
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

}

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
    NORTH,
    EAST,
    SOUTH,
    WEST,
];

pub const ORDINALS: [GridTransform; 8] = [
    NORTH,
    NORTH_EAST,
    EAST,
    SOUTH_EAST,
    SOUTH,
    SOUTH_WEST,
    WEST,
    NORTH_WEST,
];

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
