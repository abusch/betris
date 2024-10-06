use bevy::prelude::*;
use leafwing_input_manager::{
    common_conditions::action_just_pressed, plugin::InputManagerPlugin, prelude::*, Actionlike,
};

pub fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<Action>::default())
        .init_resource::<ActionState<Action>>()
        .insert_resource(Action::make_input_map())
        .add_systems(
            Update,
            toggle_pause.run_if(action_just_pressed(Action::Pause)),
        );
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Actionlike, Reflect)]
pub enum Action {
    Left,
    Right,
    RotateLeft,
    RotateRight,
    SoftDrop,
    HardDrop,
    Pause,
}

impl Action {
    pub fn make_input_map() -> InputMap<Self> {
        InputMap::new([
            (Action::Left, KeyCode::ArrowLeft),
            (Action::Right, KeyCode::ArrowRight),
            (Action::RotateLeft, KeyCode::KeyZ),
            (Action::RotateLeft, KeyCode::ArrowUp),
            (Action::RotateRight, KeyCode::KeyX),
            (Action::SoftDrop, KeyCode::ArrowDown),
            (Action::HardDrop, KeyCode::Space),
            (Action::Pause, KeyCode::KeyP),
        ])
    }
}

fn toggle_pause(mut time: ResMut<Time<Virtual>>) {
    if time.is_paused() {
        time.unpause();
    } else {
        time.pause();
    }
}
