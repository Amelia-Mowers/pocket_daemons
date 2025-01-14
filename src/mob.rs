use bevy::prelude::*;
use bevy::utils::Duration;

use crate::graph::grid_transform::*;
use crate::player::*;

use crate::map::*;

#[derive(Component, Default)]
#[require(
    Sprite,
    Transform,
    GridPosition,
    LastGridPosition,
    GridDirection,
    MovementCooldown,
    AnimationIndex,
    AnimationTimer,
)]
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

#[derive(Component, Reflect, Debug)]
pub struct AnimationIndex {
    pub current: usize,
    pub max: usize,
    pub move_only: bool,
}

impl AnimationIndex {
    fn new(max: usize, move_only: bool) -> Self {
        AnimationIndex {
            current: 0,
            max: max,
            move_only: move_only,
        }
    }
}

impl Default for AnimationIndex {
    fn default() -> Self {
        AnimationIndex::new(4, true)
    }
}

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
            mob_interact,
        ))
        .add_event::<TriggerOnMoveOntoEvent>()
        .add_event::<MobMoveEvent>()
        .add_event::<MobInteractEvent>()
        .register_type::<GridDirection>()
        .register_type::<GridPosition>()
        .register_type::<LastGridPosition>()
        .register_type::<MovementCooldown>()
        .register_type::<GridTransform>();
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

#[derive(Event, Reflect, Debug)]
pub struct TriggerEvent {
    pub triggering: Entity,
    pub triggered: Entity,
}

#[derive(Event, Reflect, Debug)]
pub struct MobInteractEvent  {
    pub entity: Entity,
}

fn mob_interact(
    query: Query<(
        &GridPosition, 
        &GridDirection,
        &MovementCooldown,
    ), With<Mob>>,
    trigger_query: Query<Entity, With<TriggerOnInteract>>,
    grid_index: Res<GridIndex>,
    mut mob_interact_events: EventReader<MobInteractEvent>,
    mut trigger_event: EventWriter<TriggerEvent>,
) {
    for event in mob_interact_events.read() {
        if let Ok((pos, dir, cooldown)) = query.get(event.entity) {
            if cooldown.finished() {
                let interact_pos = **pos + **dir;
                let grid_tile_entities = grid_index.get(&interact_pos);

                for triggered_entity in grid_tile_entities.iter().filter(|&&e| trigger_query.contains(e)) {
                    trigger_event.send(TriggerEvent { 
                        triggering: event.entity,
                        triggered: *triggered_entity,
                    });
                }
            }
        }
    }
}
