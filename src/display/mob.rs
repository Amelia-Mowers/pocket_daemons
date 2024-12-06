// use crate::player::PLAYER_SPEED; 
use crate::mob::*; 

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
    mut query: Query<(
        &GridPosition,
        &LastGridPosition,
        &MovementCooldown,
        &mut Transform,
    ), With<Mob>>,
) { 
    for (position, last, cooldown, mut transform) in &mut query {
        let current = (*transform).translation;
        let z_level = current.z;

        let last_pos: Transform = (**last).into();
        let new_pos: Transform = (**position).into();

        (*transform).translation = last_pos.translation.lerp(
            new_pos.translation, 
            cooldown.fraction()
        );

        (*transform).translation.z = z_level;
    }
}

fn update_mob_animation(
    time: Res<Time>,
    mut query: Query<(
        &GridDirection, 
        &MovementCooldown,
        &mut AnimationIndex, 
        &mut AnimationTimer,
        &mut TextureAtlas,
    )>,
) { 
    for (
        direction, 
        move_cool,
        mut index,
        mut timer,
        mut atlas,
    ) in &mut query {

        let base = direction.cardinal_index() * 4;

        timer.tick(time.delta());
        if timer.just_finished() {
            **index = (**index + 1) % 4;
        }

        if move_cool.just_finished() || !move_cool.finished() {
            (*atlas).index = base + **index;
        } else {
            (*atlas).index = base + 1;
        }
    }
}
