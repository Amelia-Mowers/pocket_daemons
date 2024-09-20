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

fn update_mob_transform(
    time: Res<Time>,
    mut query: Query<(&GridTransform, &mut Transform)>,
) { 
    for (position, mut transform) in query.iter_mut() {
        let current = (*transform).translation;
        let goal: Transform = (*position).into();

        let moving = goal.translation.distance(current) > 0.7;

        if moving {
            let diff = goal.translation - current;

            let mod_diff = 
                diff.normalize_or_zero() 
                * SCALE_FACTOR 
                * (time.delta().as_secs_f32() / PLAYER_SPEED);

            (*transform).translation = current + mod_diff;
        } else {
            (*transform).translation = goal.translation;
        }
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
        let current = (*transform).translation;
        let goal: Transform = (*position).into();
        let moving = goal.translation.distance(current) > 0.7;

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
