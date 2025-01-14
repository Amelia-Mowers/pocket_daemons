use crate::graph::grid_transform::GridTransform;
use crate::loading::TextureAssets;
use crate::{GameState, RES_HEIGHT, RES_WIDTH};
use bevy::prelude::*;
use crate::PIXEL_PERFECT_STATIC_LAYERS;
use std::collections::HashMap;
use std::collections::HashSet;
use crate::loading::FontAssets;
use bevy::sprite::*;
use crate::control::GameControlEvent;
use crate::control::GameControl;
use std::time::Duration;
use bevy::ecs::system::EntityCommands;
use crate::mob::TriggerEvent;
use crate::map::ChangeMapQueue;
use crate::loading::MapAssets;
use crate::map::ChangeMapEvent;
use crate::state_stack::StateStack;

use bevy::text::TextLayoutInfo;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(OnExit(GameState::AssetLoading), (
            init_menu,
        ))
        .add_systems(OnEnter(GameState::Menu), (
            enter_menu,
        ))
        .add_systems(OnExit(GameState::Menu), (
            exit_menu,
        ))
        .add_systems(Update, (
            update_menu_grid_index,
            update_cursor_transform.after(update_menu_grid_index),
            trigger_exit,
            rounded_center_text,
        ))
        .add_systems(Update, (
            menu_move_control,
            menu_interact_control,
            trigger_game_start,
        ).run_if(in_state(GameState::Menu)))
        .init_resource::<MenuMovementCooldown>()
        .register_type::<MenuBox>()
        .register_type::<MenuElement>()
        .register_type::<MenuCursor>();
    }
}

#[derive(Component, Default, Reflect)]
struct MenuBox {
    elements_index: HashMap<GridTransform, Entity>,
    pub grid_bounds: GridBounds,
}

#[derive(Component, Reflect)]
struct MenuElement {
    cursor_anchor: Transform,
    menu_grid_position: GridTransform,
}

#[derive(Component, Reflect)]
struct MenuCursor {
     menu_focus: Entity,
     menu_grid_position: GridTransform,
}

#[derive(Default, Reflect, Clone, Copy)]
pub struct GridBounds {
    pub min_x: i16,
    pub max_x: i16,
    pub min_y: i16,
    pub max_y: i16,
}

#[derive(Component, Default, Reflect, Clone, Copy)]
pub struct RoundedCenterText;

impl GridBounds {
    /// Create new bounds from an initial position
    pub fn from_position(pos: GridTransform) -> Self {
        Self {
            min_x: pos.x,
            max_x: pos.x,
            min_y: pos.y,
            max_y: pos.y,
        }
    }

    /// Checks if a position is inside these bounds
    pub fn includes(&self, pos: GridTransform) -> bool {
        pos.x >= self.min_x
            && pos.x <= self.max_x
            && pos.y >= self.min_y
            && pos.y <= self.max_y
    }

    /// Updates the bounds by adding a new position
    pub fn add_position(&mut self, pos: GridTransform) {
        self.min_x = self.min_x.min(pos.x);
        self.max_x = self.max_x.max(pos.x);
        self.min_y = self.min_y.min(pos.y);
        self.max_y = self.max_y.max(pos.y);
    }
}

#[derive(Component)]
struct MainMenu;

fn init_menu (
    mut commands: Commands,
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
){
    // Basic background rect
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(
            RES_WIDTH as f32, 
            RES_HEIGHT as f32,
        ))),
        MeshMaterial2d(materials.add(Color::srgb_u8(224, 240, 232))),
        Transform::from_xyz(
            RES_WIDTH as f32 / 2.0, 
            RES_HEIGHT as f32 / 2.0, 
            -1.
        ),
        PIXEL_PERFECT_STATIC_LAYERS,
        MainMenu,
    ));

    commands.spawn((
        Transform::from_xyz(
            (RES_WIDTH as f32 / 2.0 - 50.0).floor(), 
            (RES_HEIGHT as f32 / 2.0 - 5.0 + 16.0).floor(), 
            10.,
        ),
        Sprite {
            image: textures.title.clone(),
            anchor: Anchor::BottomLeft,
            ..default()
        },
        Visibility::Hidden,
        MainMenu,
        PIXEL_PERFECT_STATIC_LAYERS,
    ));

    let text_font = TextFont {
        font: fonts.font.clone(),
        font_size: 10.,
        ..Default::default()
    };

    let text_color = TextColor(Color::srgb_u8(47, 76, 64));

    // Menu box size
    // let box_size = Vec2::new(50.0, 80.0);
    let menu_box_origin = Vec2::new(
        // RES_WIDTH as f32 / 2.0 - box_size.x / 2.0,
        // RES_HEIGHT as f32 / 2.0 - box_size.y / 2.0,
        RES_WIDTH as f32 / 2.0,
        RES_HEIGHT as f32 / 2.0,
    );

    // Define our items via tuples: (label, closure).
    // The closure can be empty if no extra components are needed.
    let mut menu_items: Vec<(&str, Box<dyn FnMut(&mut EntityCommands) + Send + Sync>)> = vec![
        ("Play", Box::new(|cmd: &mut EntityCommands| {
            cmd.insert((
                TriggerOnMenuInteract,
                GameStartOnTriggered,
            ));
        })),
        ("Options", Box::new(|cmd: &mut EntityCommands| {
            cmd.insert((
                TriggerOnMenuInteract,
            ));
        })),
        ("Exit", Box::new(|cmd: &mut EntityCommands| {
            cmd.insert((
                TriggerOnMenuInteract,
                ExitOnTriggered,
            ));
        })),
    ];

    // Spawn the menu box
    let menu_box_entity = commands
        .spawn((
            Transform::from_xyz(
                menu_box_origin.x,
                menu_box_origin.y,
                0.,
            ),
            Visibility::Hidden,
            PIXEL_PERFECT_STATIC_LAYERS,
            MenuBox::default(),
            MainMenu,
        ))
        .with_children(|parent| {
            // We'll set the top-left corner inside the box for the baseline
            let base_x = 0.0;
            let base_y = 0.0;

            // Start grid_y at 0, decrement each item
            let mut grid_y = 0;

            for (label, add_extras_fn) in menu_items.iter_mut() {
                // Current grid transform for this item
                let item_grid_pos = GridTransform::new(0, grid_y);

                // We'll place each item visually by computing an offset from base_y 
                // based on the grid_y (which is negative or zero).
                //
                // e.g., if grid_y = 0 => offset = 0
                //       if grid_y = -1 => offset = -1 * 14 = -14
                let item_visual_y = base_y + (grid_y as f32 * 14.0);

                let mut entity = parent
                    .spawn((
                        Text2d::new(label.to_string()),
                        text_font.clone(),
                        text_color.clone(),
                        Anchor::BottomLeft,
                        Transform::from_xyz(
                            base_x, 
                            item_visual_y, 
                            1.0
                        ),
                        PIXEL_PERFECT_STATIC_LAYERS,
                        MenuElement {
                            cursor_anchor: Transform::from_xyz(
                                -5.0, 
                                1.0, 
                                1.0
                            ),
                            menu_grid_position: item_grid_pos,
                        },
                        MainMenu,
                        RoundedCenterText,
                    ));

                // Insert extra components if needed
                (add_extras_fn)(&mut entity);

                // Decrement grid_y so next item is one "step" below
                grid_y -= 1;
            }
        })
        .id();

    commands.spawn((
        Transform::from_xyz(
            8.,
            8.,
            10.,
        ),
        Sprite {
            image: textures.menu_pointer.clone(),
            anchor: Anchor::BottomLeft,
            ..default()
        },
        Visibility::Hidden,
        MainMenu,
        MenuCursor {
            menu_focus: menu_box_entity,
            menu_grid_position: GridTransform::ZERO,
        },
        PIXEL_PERFECT_STATIC_LAYERS,
    ));
}

fn enter_menu (
    mut main_menu_query: Query<&mut Visibility, With<MainMenu>>,
) {
    for mut visibility in &mut main_menu_query {
        *visibility = Visibility::Inherited;
    }
}

fn exit_menu (
    mut main_menu_query: Query<&mut Visibility, With<MainMenu>>,
) {
    for mut visibility in &mut main_menu_query {
        *visibility = Visibility::Hidden;
    }
}

fn update_cursor_transform(
    mut menu_cursor_query: Query<(&MenuCursor, &mut Transform), Without<MenuElement>>,
    menu_box_query: Query<&MenuBox>,
    menu_element_query: Query<(
        &GlobalTransform,
        &MenuElement,
    ), Without<MenuCursor>>,
) {
    for (cursor, mut transform) in menu_cursor_query.iter_mut() {
        let focused_box = menu_box_query
            .get(cursor.menu_focus)
            .expect("Failed to find focused menu box.");
        let target_element_entity = focused_box
            .elements_index
            .get(&cursor.menu_grid_position)
            .expect("Failed to find target element in focused box.");

        let (target_transform, element) = menu_element_query
            .get(*target_element_entity)
            .expect("Failed to get text layout and element from target.");

        transform.translation.x =
            target_transform.translation().x + element.cursor_anchor.translation.x;
        transform.translation.y =
            target_transform.translation().y + element.cursor_anchor.translation.y;
    }
}

fn update_menu_grid_index(
    // 1) Find all MenuElements whose position changed
    mut changed_positions: Query<(Entity, &Parent, &MenuElement), Changed<MenuElement>>,
    // 2) We'll need to mutate MenuBox for each parent
    mut menu_boxes: Query<(&Children, &mut MenuBox)>,
    // 3) We still need to look up MenuElement for bounding-box rebuild
    menu_element_query: Query<&MenuElement>,
) {
    // Track which MenuBox entities need their bounds recalculated
    let mut updated_box_entities: HashSet<Entity> = HashSet::new();

    // First Pass: Update elements_index directly
    for (element_entity, parent, menu_element) in &mut changed_positions {
        let menu_box_entity = parent.get();
        if let Ok((_, mut menu_box)) = menu_boxes.get_mut(menu_box_entity) {
            // 1) Remove the old position of this entity from the index
            if let Some(old_pos) = menu_box
                .elements_index
                .iter()
                .find_map(|(pos, &ent)| if ent == element_entity { Some(*pos) } else { None })
            {
                menu_box.elements_index.remove(&old_pos);
            }

            // 2) Insert the new position
            menu_box
                .elements_index
                .insert(menu_element.menu_grid_position, element_entity);

            // We know this MenuBox changed, so mark it
            updated_box_entities.insert(menu_box_entity);
        }
    }
    // Second Pass: Rebuild bounding box for each MenuBox that changed
    for box_entity in updated_box_entities {
        if let Ok((children, mut menu_box)) = menu_boxes.get_mut(box_entity) {
            // Clear existing bounds by marking an "empty" state
            let mut maybe_bounds: Option<GridBounds> = None;

            // Rebuild from child elements
            for &child in children.iter() {
                if let Ok(element) = menu_element_query.get(child) {
                    // If we haven't started the bounding box, initialize
                    if let Some(bounds) = &mut maybe_bounds {
                        bounds.add_position(element.menu_grid_position);
                    } else {
                        maybe_bounds = Some(GridBounds::from_position(element.menu_grid_position));
                    }
                }
            }

            // If no children or no valid elements, default the bounds to 0..0
            menu_box.grid_bounds = match maybe_bounds {
                Some(bounds) => bounds,
                None => GridBounds {
                    min_x: 0,
                    max_x: 0,
                    min_y: 0,
                    max_y: 0,
                },
            };
        }
    }
}

#[derive(Resource, Deref, DerefMut, Reflect, Debug)]
pub struct MenuMovementCooldown(Timer);

impl Default for MenuMovementCooldown {
    fn default() -> Self {
        let timer = Timer::new(
            Duration::from_secs_f32(0.1),
            TimerMode::Once
        );
        MenuMovementCooldown(timer)
    }
}

fn menu_move_control(
    time: Res<Time>,
    mut cooldown: ResMut<MenuMovementCooldown>,
    mut control_events: EventReader<GameControlEvent>,
    mut cursors: Query<&mut MenuCursor>,
    menu_boxes: Query<&MenuBox>,
) {
    cooldown.tick(time.delta());

    for event in control_events.read()
    .filter(|e| e.just_pressed())
    // .filter(|e| e.pressed()) // held down
    .filter(|e| e.is_movement()) {
        // if !cooldown.finished() { return }
        
        let movement = match event.control {
            GameControl::Up => GridTransform::NORTH,
            GameControl::Down => GridTransform::SOUTH,
            GameControl::Left => GridTransform::WEST,
            GameControl::Right => GridTransform::EAST,
            _ => continue,
        };

        for mut cursor in &mut cursors {
            if let Ok(menu_box) = menu_boxes.get(cursor.menu_focus) {
                let mut next_pos = cursor.menu_grid_position + movement;

                // Keep stepping in the movement direction until:
                // 1) We go out of bounds, or
                // 2) We find an element in `elements_index`
                while menu_box.grid_bounds.includes(next_pos)
                    && !menu_box.elements_index.contains_key(&next_pos)
                {
                    next_pos = next_pos + movement;
                }

                // If the final candidate is still in bounds and valid, move there.
                if menu_box.grid_bounds.includes(next_pos)
                    && menu_box.elements_index.contains_key(&next_pos)
                {
                    cursor.menu_grid_position = next_pos;
                    cooldown.reset();
                }
            }
        }
    }
}

#[derive(Component)]
pub struct TriggerOnMenuInteract;

fn menu_interact_control(
    state: Res<State<GameState>>,
    mut control_events: EventReader<GameControlEvent>,
    mut trigger_event: EventWriter<TriggerEvent>,
    trigger_query: Query<Entity, With<TriggerOnMenuInteract>>,
    cursors: Query<(Entity, &MenuCursor)>,
    menu_boxes: Query<&MenuBox>,
) {
    match control_events.read()
    .filter(|e| e.just_pressed())
    .filter(|e| e.control == GameControl::Interact)
    .nth(0) {
        Some(_) => {
            if state.is_changed() { return }
            for (cursor_entity, cursor) in &cursors {
                if let Ok(menu_box) = menu_boxes.get(cursor.menu_focus) {
                    if let Some(element) = menu_box.elements_index.get(&cursor.menu_grid_position) {
                        if trigger_query.contains(element.clone()) {
                            trigger_event.send(TriggerEvent { 
                                triggering: cursor_entity,
                                triggered: *element,
                            });
                        }
                    }
                }
            }
        },
        None => {},
    };
}

#[derive(Component)]
pub struct ExitOnTriggered;

fn trigger_exit (
    exit_query: Query<Entity, With<ExitOnTriggered>>,
    mut events: EventReader<TriggerEvent>,
    mut exit: EventWriter<AppExit>,
) {
    for event in events.read() {
        if let Ok(_) = exit_query.get(event.triggered) {
            exit.send(AppExit::Success);
        }
    }
}

#[derive(Component)]
pub struct GameStartOnTriggered;

fn trigger_game_start (
    exit_query: Query<
        Entity, 
        With<GameStartOnTriggered>
    >,
    mut events: EventReader<TriggerEvent>,
    mut main_menu_query: Query<
        &mut Visibility, 
        With<MainMenu>
    >,
    mut change_map_queue: ResMut<ChangeMapQueue>,
    map_assets: Res<MapAssets>, 
    mut next_state: ResMut<NextState<GameState>>,
    mut state_stack: ResMut<StateStack>,
) {
    for event in events.read() {
        if let Ok(_) = exit_query.get(event.triggered) {
            for mut visibility in &mut main_menu_query {
                *visibility = Visibility::Hidden;
                next_state.set(state_stack.push(GameState::Playing));
                change_map_queue.push(ChangeMapEvent{
                    map: map_assets.road.clone(), 
                    spawn: "start".to_string(),
                });
            }
        }
    }
}


fn compute_text_bounds(text_layout: &TextLayoutInfo) -> Vec2 {
    let mut max_x: f32 = 0.0;
    let mut max_y: f32 = 0.0;

    for glyph in &text_layout.glyphs {
        let right = glyph.position.x + glyph.size.x;
        let bottom = glyph.position.y + glyph.size.y;
        max_x = max_x.max(right.ceil());
        max_y = max_y.max(bottom);
    }

    Vec2::new(max_x, max_y)
}

fn rounded_center_text(
    mut commands: Commands,    
    mut query: Query<(Entity, &mut Transform, &TextLayoutInfo), With<RoundedCenterText>>,
) {
    for (entity, mut transform, text_layout) in &mut query {
        // Skip if there are no glyphs
        if text_layout.glyphs.is_empty() {
            continue;
        }

        // Compute the bounding box of the text
        let bounds = compute_text_bounds(text_layout);
        let half_bounds = bounds * 0.5;

        // Subtract half the width/height from the transform
        // and floor it inline
        transform.translation.x = (transform.translation.x - half_bounds.x).floor();
        transform.translation.y = (transform.translation.y - half_bounds.y).floor();

        // Remove the marker so we only do this once
        commands.entity(entity).remove::<RoundedCenterText>();
    }
}
