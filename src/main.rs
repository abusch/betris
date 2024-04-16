use bevy::{prelude::*, window::close_on_esc};
use game::GamePlugin;
use splash::SplashPlugin;

mod game;
mod pieces;
mod splash;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .add_plugins((SplashPlugin, GamePlugin))
        .run();
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Pos {
    pub x: isize,
    pub y: isize,
}

impl Pos {
    pub const fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }

    /// Return the position to the left of the current one
    #[must_use]
    pub fn left(&self) -> Self {
        Self {
            x: self.x - 1,
            ..*self
        }
    }

    /// Return the position to the right of the current one
    #[must_use]
    pub fn right(&self) -> Self {
        Self {
            x: self.x + 1,
            ..*self
        }
    }

    /// Return the position below the current one
    #[must_use]
    pub fn down(&self) -> Self {
        Self {
            y: self.y - 1,
            ..*self
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
