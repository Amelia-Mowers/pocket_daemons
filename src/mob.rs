use crate::GameState;
use bevy::prelude::*;
use bevy::sprite::*;
use bevy::utils::Duration;

use crate::graph::grid_transform::*;
use crate::player::*;

use crate::map::*;

#[derive(Component, Default)]
pub struct Mob;

#[derive(Component, Deref, DerefMut, Reflect, Debug, Default)]
pub struct GridPosition(pub GridTransform);

#[derive(Component, Deref, DerefMut, Reflect, Debug, Default)]
pub struct LastGridPosition(pub GridTransform);

#[derive(Component, Deref, DerefMut, Reflect, Debug)]
pub struct GridDirection(pub GridTransform);

impl Default for GridDirection {
    fn default() -> Self {
        GridDirection(GridTransform::SOUTH)
    }
}

#[derive(Component, Deref, DerefMut, Reflect, Debug, Default)]
pub struct MoveTo(pub Option<GridTransform>);

#[derive(Component, Deref, DerefMut, Reflect, Debug, Default)]
pub struct AnimationIndex(usize);

#[derive(Component, Deref, DerefMut, Reflect, Debug)]
pub struct AnimationTimer(Timer);

impl Default for AnimationTimer {
    fn default() -> Self {
        AnimationTimer(Timer::from_seconds(
            0.1, 
            TimerMode::Repeating
        ))
    }
}

#[derive(Component, Deref, DerefMut, Reflect, Debug)]
pub struct MovementCooldown(Timer);

impl Default for MovementCooldown {
    fn default() -> Self {
        let mut timer = Timer::new(
            Duration::from_secs_f32(0.4),
            TimerMode::Once
        );
        timer.finish();
        MovementCooldown(timer)
    }
}

pub trait TimerFinish {
    fn finish(&mut self);
}
impl TimerFinish for Timer {
    fn finish(&mut self) {
        let duration = self.duration();
        self.tick(duration);
        self.tick(duration);
    }
}

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            move_mob.after(player_move_control),
        ).run_if(in_state(GameState::Playing)))
        .add_event::<TriggerOnMoveOntoEvent>()
        .add_event::<MobMoveEvent>()
        .register_type::<GridDirection>()
        .register_type::<GridPosition>()
        .register_type::<LastGridPosition>()
        .register_type::<MoveTo>()
        .register_type::<MovementCooldown>()
        .register_type::<GridTransform>();
    }
}

#[derive(Bundle)]
pub struct MobBundle {
    pub mob: Mob,
    pub grid_position: GridPosition,
    pub last_grid_position: LastGridPosition,
    pub move_to: MoveTo,
    pub grid_direction: GridDirection,
    pub movement_cooldown: MovementCooldown,
    pub texture_atlas: TextureAtlas,
    pub animation_index: AnimationIndex,
    pub animation_timer: AnimationTimer,

    pub sprite: Sprite,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub texture: Handle<Image>,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub index_grid_position: IndexGridPosition,
}

impl Default for MobBundle {
    fn default() -> Self {
        Self {
            mob: Default::default(),
            grid_position: Default::default(),
            last_grid_position: Default::default(),
            move_to: Default::default(),
            grid_direction: Default::default(),
            movement_cooldown: Default::default(),
            texture_atlas: Default::default(),
            animation_index: Default::default(),
            animation_timer: Default::default(),

            sprite: Sprite {
                anchor: Anchor::Custom(Vec2::new(-0.5, -(14.0/16.0))),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3 { x: 0., y: 0., z: 1. }),
            global_transform: Default::default(),
            texture: Default::default(),
            visibility: Default::default(),
            inherited_visibility: Default::default(),
            view_visibility: Default::default(),
            index_grid_position: IndexGridPosition,
        }
    }
}

#[derive(Event, Reflect, Debug)]
pub struct TriggerOnMoveOntoEvent {
    pub moved: Entity,
    pub triggered: Entity,
}

#[derive(Event, Reflect, Debug)]
pub struct MobMoveEvent  {
    pub entity: Entity,
    pub movement: GridTransform,
}

fn move_mob(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut GridPosition, 
        &mut LastGridPosition, 
        &mut GridDirection,
        &mut MovementCooldown,
    ), With<Mob>>,
    block_query: Query<Entity, With<BlocksWalking>>,
    trigger_query: Query<Entity, With<TriggerOnMoveOnto>>,
    mut grid_index: ResMut<GridIndex>,
    mut mob_move_events: EventReader<MobMoveEvent>,
    mut move_trigger_event: EventWriter<TriggerOnMoveOntoEvent>,
) {
    for event in mob_move_events.read() {
        if let Ok((mob_entity, mut pos, mut last_pos, mut dir, mut cooldown)) = query.get_mut(event.entity) {
            if cooldown.finished() {
                let new_pos = **pos + event.movement;
                **dir = event.movement;

                let grid_tile_entities = grid_index.get(&new_pos);
                if !grid_tile_entities.iter().any(|&e| block_query.contains(e)) {
                    **last_pos = pos.0;
                    **pos = new_pos;
                    cooldown.reset();

                    for triggered_entity in grid_tile_entities.iter()
                        .filter(|&&e| trigger_query.contains(e)) {
                        move_trigger_event.send(TriggerOnMoveOntoEvent { 
                            moved: mob_entity,
                            triggered: *triggered_entity,
                        });
                    }

                    grid_index.update(mob_entity, new_pos);
                }
            }
        }
    }

    for (_, _, _, _, mut cooldown)in &mut query {
        (**cooldown).tick(time.delta());
    }
}
