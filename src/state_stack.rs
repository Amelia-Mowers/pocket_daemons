use bevy::prelude::*;
use crate::GameState;

pub struct StateStackPlugin;

impl Plugin for StateStackPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<StateStack>()
        .register_type::<StateStack>();
    }
}

#[derive(Resource, Reflect, Debug, Default)]
pub struct StateStack {
    stack: Vec<GameState>,
}

impl StateStack {
    pub fn back(&mut self) -> GameState {
        self.stack.pop();
        self.stack.last().unwrap().clone()
    }

    pub fn push(&mut self, input: GameState) -> GameState {
        self.stack.push(input.clone());
        input
    }
}

