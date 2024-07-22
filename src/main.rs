use bevy::prelude::*;
use game::GamePlugin;
use leafwing_input_manager::{
    action_state::ActionState, input_map::InputMap, plugin::InputManagerPlugin, Actionlike,
};
use splash::SplashPlugin;

mod game;
mod pieces;
mod pos;
mod splash;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<Action>::default())
        .init_resource::<ActionState<Action>>()
        .insert_resource(Action::make_input_map())
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Actionlike, Reflect)]
pub enum Action {
    Left,
    Right,
    RotateLeft,
    RotateRight,
    Down,
    Drop,
}

impl Action {
    pub fn make_input_map() -> InputMap<Self> {
        InputMap::new([
            (Action::Left, KeyCode::ArrowLeft),
            (Action::Right, KeyCode::ArrowRight),
            (Action::RotateLeft, KeyCode::KeyZ),
            (Action::RotateRight, KeyCode::KeyX),
            (Action::Down, KeyCode::ArrowDown),
            (Action::Drop, KeyCode::Space),
        ])
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
