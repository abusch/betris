use bevy::{
    dev_tools::{
        diagnostics_overlay::{DiagnosticsOverlay, DiagnosticsOverlayPlugin},
        states::log_transitions,
    },
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    input::common_conditions::input_just_pressed,
    prelude::*,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::ResourceInspectorPlugin};

use crate::{
    AppSystems,
    game::{GameState, Phase},
    screen::Screen,
};

#[derive(Resource)]
struct DebugInfo(bool);

pub fn plugin(app: &mut App) {
    app.insert_resource(DebugInfo(false))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(EntityCountDiagnosticsPlugin::default())
        .add_plugins(SystemInformationDiagnosticsPlugin)
        .add_plugins((
            EguiPlugin::default(),
            ResourceInspectorPlugin::<GameState>::default().run_if(is_debug),
            DiagnosticsOverlayPlugin,
        ))
        .add_systems(
            Update,
            (
                toggle_debug.run_if(input_just_pressed(KeyCode::KeyD)),
                close_on_esc.in_set(AppSystems::RecordInput),
                log_transitions::<Screen>,
                log_transitions::<Phase>,
            ),
        );
}

fn is_debug(debug_info: Res<DebugInfo>) -> bool {
    debug_info.0
}

fn toggle_debug(
    mut commands: Commands,
    mut debug_info: ResMut<DebugInfo>,
    overlays: Query<Entity, With<DiagnosticsOverlay>>,
) {
    debug_info.0 = !debug_info.0;

    if debug_info.0 {
        commands.spawn(DiagnosticsOverlay::fps());
    } else {
        for o in overlays {
            commands.entity(o).despawn();
        }
    }
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
