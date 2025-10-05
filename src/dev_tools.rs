use bevy::{
    dev_tools::{
        fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
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
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    enabled: false,
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 16.0,
                        ..default()
                    },
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: false,
                        // The minimum acceptable fps
                        min_fps: 30.0,
                        // The target fps
                        target_fps: 60.0,
                    },
                    ..default()
                },
            },
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

fn toggle_debug(mut debug_info: ResMut<DebugInfo>, mut fps_config: ResMut<FpsOverlayConfig>) {
    debug_info.0 = !debug_info.0;

    fps_config.enabled = debug_info.0;
    fps_config.frame_time_graph_config.enabled = debug_info.0;
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
