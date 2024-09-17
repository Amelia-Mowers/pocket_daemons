use crate::display::scale::SCALE_FACTOR;
use crate::graph::grid_transform::GridTransform;

use crate::player::PLAYER_SPEED; 

use bevy::{
    prelude::*,
};

pub struct MobDisplayPlugin;

impl Plugin for MobDisplayPlugin { 
    fn build (&self, app: &mut App) {
        app
            .add_systems(Update, (
                update_mob_transform,
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

        if goal.translation.distance(current) > 1.0 {
            let diff = goal.translation - current;

            let mod_diff = 
                diff.normalize_or_zero() 
                * SCALE_FACTOR 
                * (time.delta().as_secs_f32() / PLAYER_SPEED);

            (*transform).translation = current + mod_diff;
        }
    }
}
