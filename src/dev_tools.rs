use bevy::{dev_tools::states::log_transitions, prelude::*};
use iyes_perf_ui::{entries::PerfUiBundle, PerfUiPlugin};

use crate::{game::Phase, screen::Screen, AppSet};

pub fn plugin(app: &mut App) {
    app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                close_on_esc.in_set(AppSet::RecordInput),
                log_transitions::<Screen>,
                log_transitions::<Phase>,
            ),
        );
}

fn setup(mut commands: Commands) {
    commands.spawn(PerfUiBundle::default());
}

fn close_on_esc(
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
