use bevy::prelude::*;

use super::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), enter_playing);
    app.add_systems(OnExit(Screen::Playing), exit_playing);
}

fn enter_playing(mut _cmd: Commands) {}

fn exit_playing(mut _cmd: Commands) {}
