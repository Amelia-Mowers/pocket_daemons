#![allow(clippy::type_complexity)]

mod audio;
mod loading;
mod menu;
mod player;
mod graph;
mod display;
mod map;
mod helpers;
mod mob;
mod control;
mod text_loading;
mod dialog;
mod state_stack;

use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;
use crate::player::Player;
use crate::map::MapPlugin;
use crate::display::mob::MobDisplayPlugin;
use crate::control::ControlPlugin;
use crate::mob::MobPlugin;
use crate::text_loading::TextLoadingPlugin;
use crate::dialog::DialogPlugin;
use crate::state_stack::StateStackPlugin;

use bevy_inspector_egui::quick::WorldInspectorPlugin;

// use bevy::app::App;
use bevy::{
    // prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    window::WindowResized,
};
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Reflect, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    TextLoading,
    AssetLoading,
    // During this State the actual game logic is executed
    Playing,
    Dialog,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

/// In-game resolution width.
pub const RES_WIDTH: u32 = 160;

/// In-game resolution height.
pub const RES_HEIGHT: u32 = 144;

/// Default render layers for pixel-perfect rendering.
/// You can skip adding this component, as this is the default.
const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);

const PIXEL_PERFECT_STATIC_LAYERS: RenderLayers = RenderLayers::layer(1);

/// Render layers for high-resolution rendering.
const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(2);

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
        .init_state::<GameState>()
        .add_plugins((
            LoadingPlugin,
            MenuPlugin,
            InternalAudioPlugin,
            PlayerPlugin,
            MobPlugin,
            MobDisplayPlugin,
            MapPlugin,
            ControlPlugin,
            TextLoadingPlugin,
            DialogPlugin,
            StateStackPlugin,
            WorldInspectorPlugin::new(),
        ))
        .add_systems(Startup, (
            setup_camera, 
            // setup_sprite,
        ))
        .add_systems(Update, (
            fit_canvas,
            // rotate,
            camera_follow_player,
        ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}

/// Low-resolution texture that contains the pixel-perfect world.
/// Canvas itself is rendered to the high-resolution world.
#[derive(Component)]
struct Canvas;

/// Camera that renders the pixel-perfect world to the [`Canvas`].
#[derive(Component)]
struct InGameCamera;

/// Camera that renders the [`Canvas`] (and other graphics on [`HIGH_RES_LAYERS`]) to the screen.
#[derive(Component)]
struct OuterCamera;

fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas_size = Extent3d {
        width: RES_WIDTH,
        height: RES_HEIGHT,
        ..default()
    };

    // this Image serves as a canvas representing the low-resolution game screen
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    let mut static_canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    canvas.resize(canvas_size);
    static_canvas.resize(canvas_size);

    let image_handle = images.add(canvas);
    let static_image_handle = images.add(static_canvas);

    // this camera renders whatever is on `PIXEL_PERFECT_LAYERS` to the canvas
    commands.spawn((
        Camera2d, 
        Camera {
            // render before the "main pass" camera
            order: -2,
            target: RenderTarget::Image(image_handle.clone()),
            ..default()
        },
        Msaa::Off,
        InGameCamera,
        PIXEL_PERFECT_LAYERS,
    ));

    commands.spawn((
        Camera2d, 
        Camera {
            // render before the "main pass" camera
            order: -1,
            target: RenderTarget::Image(static_image_handle.clone()),
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        Transform::from_xyz(
            RES_WIDTH as f32 / 2., 
            RES_HEIGHT as f32 / 2., 
            10.,
        ),
        Msaa::Off,
        PIXEL_PERFECT_STATIC_LAYERS,
    ));

    // spawn the canvas
    commands.spawn((
        Sprite {
            image: image_handle,
            ..default()
        },
        Canvas,
        HIGH_RES_LAYERS,
    ));

    // spawn the canvas
    commands.spawn((
        Sprite {
            image: static_image_handle,
            ..default()
        },
        Transform::from_xyz(0., 0., 10.),
        Canvas,
        HIGH_RES_LAYERS,
    ));

    // the "outer" camera renders whatever is on `HIGH_RES_LAYERS` to the screen.
    // here, the canvas and one of the sample sprites will be rendered by this camera
    commands.spawn((
        Camera2d, 
        Msaa::Off,
        OuterCamera, 
        HIGH_RES_LAYERS,
    ));
}

fn camera_follow_player(
    p: Query<&Transform, (With<Player>, Without<InGameCamera>)>,
    mut c: Query<&mut Transform, (With<InGameCamera>, Without<Player>)>,
) {
    if let Ok(mut camera_transform) = c.get_single_mut() {
        if let Ok(player_transform) = p.get_single() {
            // *camera_transform = *player_transform;
            *camera_transform = (*player_transform).with_translation(
                (*player_transform).translation +
                Vec3 {
                    x: 8.,
                    y: 8.,
                    z: 0.,
                }
            );
        }
    }
}

/// Scales camera projection to fit the window (integer multiples only).
fn fit_canvas(
    mut resize_events: EventReader<WindowResized>,
    mut projections: Query<&mut OrthographicProjection, With<OuterCamera>>,
) {
    for event in resize_events.read() {
        let h_scale = event.width / RES_WIDTH as f32;
        let v_scale = event.height / RES_HEIGHT as f32;
        let mut projection = projections.single_mut();
        projection.scale = 1. / h_scale.min(v_scale).round();
    }
}
