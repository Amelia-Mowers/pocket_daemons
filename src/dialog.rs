use bevy::prelude::*;
use crate::GameState;
use bevy::sprite::*;

use crate::text_loading::Dialog;
use crate::RES_WIDTH;
use crate::PIXEL_PERFECT_STATIC_LAYERS;
use crate::loading::TextureAssets;
use crate::loading::FontAssets;

use crate::control::GameControlEvent;
use crate::control::GameControl;
use crate::state_stack::StateStack;

use bevy::text::TextBounds;
use bevy::text::LineBreak;

pub struct DialogPlugin;

impl Plugin for DialogPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnExit(GameState::AssetLoading), (
                init_dialog,
            ))
            .add_systems(OnEnter(GameState::Dialog), (
                enter_dialog,
            ))
            // Update systems for dialog state
            .add_systems(Update, (
                dialog_control,
                change_page,
                update_current_page_text, // Tick timer and update index
                stream_text,              // Update displayed text if changed
            ).chain().run_if(in_state(GameState::Dialog)))
            .add_systems(OnExit(GameState::Dialog), (
                exit_dialog,
            ))
            .init_resource::<CurrentDialog>()
            .init_resource::<CurrentPageIndex>()
            .init_resource::<CurrentPageText>()
            .init_resource::<TextRevealTimer>()
            .add_event::<PageEvent>();
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CurrentDialog(pub Option<Dialog>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CurrentPageIndex(usize);

#[derive(Event, Default, Deref, DerefMut)]
pub struct PageEvent(usize);

#[derive(Resource, Default, Debug)]
pub struct CurrentPageText {
    pub full_text: String,
    pub current_index: usize,
}

#[derive(Resource, Deref, DerefMut, Reflect, Debug)]
pub struct TextRevealTimer(Timer);

impl Default for TextRevealTimer {
    fn default() -> Self {
        TextRevealTimer(Timer::from_seconds(
            0.05, // adjust speed as desired
            TimerMode::Repeating
        ))
    }
}

#[derive(Component)]
struct DialogBox;

#[derive(Component)]
struct DialogText;

fn init_dialog(
    mut commands: Commands,
    textures: Res<TextureAssets>,
    fonts: Res<FontAssets>,
) {
    let box_size = Vec2::new(RES_WIDTH as f32 - 16., 48.);
    let text_box_size = Vec2::new(box_size.x - 4., box_size.y - 4.);

    commands.spawn((
        Name::new("Dialog Box".to_string()),
        Transform::from_xyz(
            8.,
            8.,
            0.,
        ),
        Sprite {
            image: textures.dialog_box.clone(),
            custom_size: Some(box_size),
            anchor: Anchor::BottomLeft,
            image_mode: SpriteImageMode::Sliced(TextureSlicer {
                border: BorderRect::square(8.),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 1.0,
            }),
            ..default()
        },
        Visibility::Hidden,
        PIXEL_PERFECT_STATIC_LAYERS,
        DialogBox,
    )).with_children(|builder| {
        builder.spawn((
            Name::new("Dialog Text".to_string()),
            Text2d::new("".to_string()),
            TextColor(Color::srgb_u8(47, 76, 64)),
            TextFont {
                font: fonts.font.clone(),
                font_size: 10.,
                ..Default::default()
            },
            TextLayout {
                justify: JustifyText::Left,
                linebreak: LineBreak::NoWrap,
            },
            Anchor::TopLeft,
            TextBounds { 
                width: Some(text_box_size.x), 
                height: Some(text_box_size.y), 
            },
            Transform::from_xyz(
                8.,
                box_size.y - 2.,
                10.,
            ),
            PIXEL_PERFECT_STATIC_LAYERS,
            DialogText,
        ));
    });
}

fn enter_dialog(
    mut events: EventWriter<PageEvent>,
    mut dialog_box_query: Query<&mut Visibility, With<DialogBox>>,
) {
    for mut visibility in &mut dialog_box_query {
        *visibility = Visibility::Inherited;
    }
    events.send(PageEvent(0));
}

fn change_page(
    mut events: EventReader<PageEvent>,
    mut current_page_index: ResMut<CurrentPageIndex>,
    mut current_page_text: ResMut<CurrentPageText>,
    current_dialog: Res<CurrentDialog>,
) {
    for event in events.read() {
        *current_page_index = CurrentPageIndex(**event);
        if let Some(dialog) = (*current_dialog).as_ref() {
            let full_text = dialog[**event].spans.iter()
                .map(|x| x.text.clone())
                .collect::<Vec<_>>()
                .join("");

            current_page_text.full_text = full_text;
            current_page_text.current_index = 0;
        }
    }
}

fn exit_dialog(
    mut dialog_box_query: Query<&mut Visibility, With<DialogBox>>,
) {
    for mut visibility in &mut dialog_box_query {
        *visibility = Visibility::Hidden;
    }
}

/// System that updates the current page text based on the timer.
/// This increments the current_page_text.current_index when the timer finishes.
fn update_current_page_text(
    time: Res<Time>,
    mut timer: ResMut<TextRevealTimer>,
    mut current_page_text: ResMut<CurrentPageText>,
) {
    let full_len = current_page_text.full_text.len();
    if current_page_text.current_index < full_len {
        timer.tick(time.delta());
        if timer.just_finished() {
            // Reveal one more character
            current_page_text.current_index += 1;
        }
    }
}

/// System that updates the dialog text if the CurrentPageText resource changed.
fn stream_text(
    mut dialog_text_query: Query<&mut Text2d, With<DialogText>>,
    current_page_text: Res<CurrentPageText>,
) {
    // Only update if the resource has changed this frame.
    if !current_page_text.is_changed() {
        return;
    }

    let displayed_text = &current_page_text.full_text[..current_page_text.current_index];
    for mut text in &mut dialog_text_query {
        *text = Text2d::new(displayed_text);
    }
}

pub fn dialog_control(
    state: Res<State<GameState>>,
    mut control_events: EventReader<GameControlEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut state_stack: ResMut<StateStack>,
    current_dialog: Res<CurrentDialog>,
    current_page_index: Res<CurrentPageIndex>,
    mut page_events: EventWriter<PageEvent>,
    mut current_page_text: ResMut<CurrentPageText>,
) {
    if let Some(_) = control_events.read()
        .filter(|e| e.just_pressed())
        .find(|e| e.control == GameControl::Interact)
    {
        if !state.is_changed() {
            if let Some(dialog) = (*current_dialog).as_ref() {
                let total_pages = dialog.len();
                let current_index = current_page_index.0;
                let full_text_len = current_page_text.full_text.len();
                let current_len = current_page_text.current_index;

                // If text not fully revealed yet, reveal it instantly
                if current_len < full_text_len {
                    current_page_text.current_index = full_text_len;
                    // current_page_text is now changed, stream_text will update on next frame
                    return;
                }

                // If fully revealed, move to next page or close dialog
                if current_index < total_pages - 1 {
                    page_events.send(PageEvent(current_index + 1));
                } else {
                    next_state.set(state_stack.back());
                }
            } else {
                // If there's no dialog, just close.
                    next_state.set(state_stack.back());
            }
        }
    }
}
