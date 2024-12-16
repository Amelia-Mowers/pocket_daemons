use crate::GameState;
use bevy::prelude::*;
use serde::Deserialize;
use sys_locale::get_locale;
use thiserror::Error;
use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::asset::AsyncReadExt;

pub struct TextLoadingPlugin;

impl Plugin for TextLoadingPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Update, (
            spawn_game_text_resource_and_advance_state,
        ).run_if(in_state(GameState::TextLoading)))
        .init_asset::<GameText>()
        .init_asset_loader::<GameTextAssetLoader>()
        .init_resource::<TempGameTextHandle>();
    }
}

fn spawn_game_text_resource_and_advance_state (
    mut commands: Commands, 
    mut next_state: ResMut<NextState<GameState>>,
    mut game_text_server: ResMut<Assets<GameText>>,
    game_text_handle: Res<TempGameTextHandle>,
) {
    if let Some(game_text) = game_text_server.remove(&**game_text_handle) {
        commands.insert_resource(game_text);
        commands.remove_resource::<TempGameTextHandle>();
        next_state.set(GameState::AssetLoading);
    }
}

// fn default_false() -> bool { false }
fn default_speed() -> f32 { 1.0 }
fn default_zero() -> u64 { 0 }

#[derive(Resource, Reflect, Asset, Debug, Deserialize)]
pub struct GameText {
    pub string_test: String,
    pub dialog_test: Dialog,
}

#[derive(Debug, Reflect, Deserialize, Deref, DerefMut, Clone)]
pub struct Dialog(Vec<Page>);

#[derive(Debug, Reflect, Deserialize, Clone)]
pub struct Page {
    pub speaker: String,
    pub mood: String,
    pub spans: Vec<Span>,
}

#[derive(Debug, Reflect, Deserialize, Clone)]
pub struct Span {
    pub text: String,

    #[serde(default = "default_speed")]
    pub speed: f32,

    // A pause before displaying this span, in milliseconds
    #[serde(default = "default_zero")]
    pub pausebefore: u64,
}

#[derive(Resource, Deref, DerefMut)]
pub struct TempGameTextHandle(Handle<GameText>);

impl FromWorld for TempGameTextHandle {
    fn from_world(world: &mut World) -> Self {
        let locale = get_locale().unwrap_or_else(|| String::from("en-US"));
        let path = format!("locales/{}.ron", locale);
        let asset_server = world.resource::<AssetServer>();
        TempGameTextHandle(asset_server.load(path))
    }
}

#[derive(Default)]
struct GameTextAssetLoader;

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
enum GameTextAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for GameTextAssetLoader {
    type Asset = GameText;
    type Settings = ();
    type Error = GameTextAssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let custom_asset = ron::de::from_bytes::<GameText>(&bytes)?;
        Ok(custom_asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}


