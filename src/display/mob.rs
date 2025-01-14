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
    ), (
        With<Mob>,
        Without<InitGridPosition>,
    )>,
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
        &mut Sprite,
    )>,
) { 
    for (
        direction, 
        move_cool,
        mut index,
        mut timer,
        mut sprite,
    ) in &mut query {

        let max = (*index).max as usize;

        let base = direction.cardinal_index() * max;

        timer.tick(time.delta());
        if timer.just_finished() {
            (*index).current = ((*index).current + 1) % (*index).max;
        }

        let current = index.current as usize;

        if let Some(ref mut atlas) = &mut sprite.texture_atlas {
            if move_cool.just_finished() || !move_cool.finished() || !(*index).move_only {
                atlas.index = base + current;
            } else {
                atlas.index = base + 1;
            }
        }
    }
}
