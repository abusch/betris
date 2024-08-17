use bevy::prelude::*;
use leafwing_input_manager::{plugin::InputManagerPlugin, prelude::*, Actionlike};

pub fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<Action>::default())
        .init_resource::<ActionState<Action>>()
        .insert_resource(Action::make_input_map());
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
