use bevy::prelude::*;
use game::GamePlugin;
use splash::SplashPlugin;

mod game;
mod pieces;
mod pos;
mod splash;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .enable_state_scoped_entities::<AppState>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .add_plugins((SplashPlugin, GamePlugin))
        .run();
}

pub fn close_on_esc(
    mut commands: Commands,
    windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (id, win) in windows.iter() {
        if win.focused && input.just_pressed(KeyCode::Escape) {
            commands.entity(id).despawn();
        }
    }
}

#[derive(States, PartialEq, Eq, Clone, Copy, Default, Debug, Hash)]
pub enum AppState {
    #[default]
    Splash,
    MainMenu,
    InGame,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
