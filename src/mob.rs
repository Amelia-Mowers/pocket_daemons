use bevy::prelude::*;
use bevy::utils::Duration;

use crate::graph::grid_transform::*;
use crate::player::*;

use crate::GameState;

use crate::map::*;


#[derive(Component, Default, Reflect)]
#[reflect(Component, Default)]
#[require(
    Sprite,
    Transform,
    GridPosition,
    LastGridPosition,
    GridDirection,
    MovementCooldown,
    AnimationIndex,
    AnimationTimer,
    InitGridPosition,
)]
pub struct Mob;

#[derive(Component, Default, Reflect)]
pub struct InitGridPosition;

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
#[reflect(Component, Default)]
#[require(
    Sprite,
    GridDirection,
    MovementCooldown,
    AnimationTimer,
)]
pub struct AnimationIndex {
    pub current: u16,
    pub max: u16,
    pub move_only: bool,
}

impl AnimationIndex {
    pub fn new(max: u16, move_only: bool) -> Self {
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
pub struct SightTriggerCooldown(Timer);

impl Default for SightTriggerCooldown {
    fn default() -> Self {
        SightTriggerCooldown(Timer::from_seconds(
            2.0, 
            TimerMode::Once,
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
            init_grid_from_transform,
        ))
        .add_systems(Update, (
            trigger_on_see_player,
        ).run_if(in_state(GameState::Playing)))
        .add_event::<TriggerOnMoveOntoEvent>()
        .add_event::<MobMoveEvent>()
        .add_event::<MobInteractEvent>()
        .register_type::<Mob>()
        .register_type::<AnimationIndex>()
        .register_type::<GridDirection>()
        .register_type::<GridPosition>()
        .register_type::<LastGridPosition>()
        .register_type::<MovementCooldown>()
        .register_type::<TriggerOnSeePlayer>()
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

fn init_grid_from_transform(
    mut commands: Commands,    
    mut query: Query<(
        Entity,
        &mut GridPosition,
        &Transform,
    ), (
        With<InitGridPosition>,
    )>,
) { 
    for (entity, mut grid_position, transform) in &mut query {
        *grid_position = GridPosition((*transform).into());
        commands.entity(entity).remove::<InitGridPosition>();
    }
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

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default)]
#[require(SightTriggerCooldown)]
struct TriggerOnSeePlayer;

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default)]
struct BlocksSight;

fn trigger_on_see_player(
    time: Res<Time>,
    mut trigger_query: Query<(
        Entity, 
        &GridPosition, 
        &GridDirection,
        &mut SightTriggerCooldown,
    ), With<TriggerOnSeePlayer>>,
    player_query: Query<Entity, With<Player>>,
    blocks_sight_query: Query<(), With<BlocksSight>>,
    grid_index: Res<GridIndex>,
    mut trigger_events: EventWriter<TriggerEvent>,
) {
    for (triggered, grid_pos, grid_dir, mut cooldown) in &mut trigger_query {
        (**cooldown).tick(time.delta());
        if !cooldown.finished() {
            break;
        }
        for step in 1..=16 {
            let check_pos = **grid_pos + (**grid_dir).mult(step);
            let occupants = grid_index.get(&GridPosition(check_pos));

            // Check each occupant for blocking or a player
            let mut blocked = false;
            let mut found_player = false;

            for &occupant in occupants {
                // If this occupant blocks sight, stop scanning entirely.
                if blocks_sight_query.contains(occupant) {
                    blocked = true;
                    break;
                }

                // If this occupant is a player, we send the event and stop.
                if player_query.contains(occupant) {
                    warn!("PLAYER DETECTED");
                    cooldown.reset();
                    trigger_events.send(TriggerEvent {
                        triggering: occupant,
                        triggered: triggered,
                    });
                    found_player = true;
                    break;
                }
            }

            // If vision is blocked or we've found a player, stop scanning further.
            if blocked || found_player {
                break;
            }
        }
    }
}
