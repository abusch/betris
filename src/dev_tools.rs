use bevy::{dev_tools::states::log_transitions, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::{ResourceInspectorPlugin, StateInspectorPlugin},
};
use iyes_perf_ui::{PerfUiPlugin, prelude::PerfUiDefaultEntries};

use crate::{
    AppSystems,
    game::{GameState, Phase},
    screen::Screen,
};

pub fn plugin(app: &mut App) {
    app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins((
            EguiPlugin {
                enable_multipass_for_primary_context: false,
            },
            StateInspectorPlugin::<Phase>::default(),
            ResourceInspectorPlugin::<GameState>::default(),
        ))
        .add_plugins(PerfUiPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                close_on_esc.in_set(AppSystems::RecordInput),
                log_transitions::<Screen>,
                log_transitions::<Phase>,
            ),
        );
}

fn setup(mut commands: Commands) {
    commands.spawn(PerfUiDefaultEntries::default());
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
