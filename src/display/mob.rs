use crate::display::scale::SCALE_FACTOR;
use crate::graph::grid_transform::GridTransform;

// use crate::player::PLAYER_SPEED; 
use crate::player::*; 

use bevy::{
    prelude::*,
};

pub struct MobDisplayPlugin;

impl Plugin for MobDisplayPlugin { 
    fn build (&self, app: &mut App) {
        app
            .add_systems(Update, (
                update_mob_transform,
                update_mob_animation,
            ));
    }

}

const MOVE_CUTOFF: f32 = 0.8;

fn update_mob_transform(
    time: Res<Time>,
    mut query: Query<(&GridTransform, &mut Transform)>,
) { 
    for (position, mut transform) in query.iter_mut() {
        let current = (*transform).translation;
        let goal: Transform = (*position).into();

        let z_level = current.z;

        (*transform).translation = 
            current.move_towards(
                goal.translation, 
                (SCALE_FACTOR * (time.delta().as_secs_f32() / PLAYER_SPEED))
            );

        
        (*transform).translation.z = z_level;
    }
}

fn update_mob_animation(
    time: Res<Time>,
    mut query: Query<(
        &GridDirection, 
        &GridTransform, 
        &mut Transform, 
        &mut AnimationIndex, 
        &mut AnimationTimer,
        &mut TextureAtlas,
    )>,
) { 
    for (
        direction, 
        position,
        mut transform,
        mut index,
        mut timer,
        mut atlas,
    ) in query.iter_mut() {
        let current = (*transform).translation.with_z(0.0);
        let goal: Transform = (*position).into();

        let moving = goal.translation.distance(current) > MOVE_CUTOFF;

        let base = direction.cardinal_index() * 4;

        timer.tick(time.delta());
        if timer.just_finished() {
            **index = (**index + 1) % 4;
        }

        if moving {
            (*atlas).index = base + **index;
        } else {
            (*atlas).index = base + 1;
        }
    }
}
