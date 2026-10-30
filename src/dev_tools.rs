use bevy::{
    dev_tools::{
        diagnostics_overlay::{
            DiagnosticsOverlay, DiagnosticsOverlayPlane, DiagnosticsOverlayPlugin,
        },
        states::log_transitions,
    },
    diagnostic::{
        EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    feathers::{
        FeathersPlugins,
        containers::{pane, pane_body, pane_header, subpane, subpane_body, subpane_header},
        controls::{
            FeathersNumberInput, HardLimit, NumberInputPrecision, NumberInputStep, NumberInputValue,
        },
        dark_theme::create_dark_theme,
        display::{caption, label, label_small},
        theme::{ThemeBackgroundColor, UiTheme},
        tokens,
    },
    input::common_conditions::input_just_pressed,
    input_focus::tab_navigation::TabGroup,
    post_process::effect_stack::{ChromaticAberration, LensDistortion},
    prelude::*,
    ui_widgets::ValueChange,
};

use crate::{AppSystems, Scanlines, game::Phase, screen::Screen};

#[derive(Resource)]
struct DebugState {
    diagnostics_enabled: bool,
    debug_ui_enabled: bool,
}

pub fn plugin(app: &mut App) {
    app.insert_resource(DebugState {
        diagnostics_enabled: false,
        debug_ui_enabled: false,
    })
    // Turn on some diagnostics
    .add_plugins((
        FrameTimeDiagnosticsPlugin::default(),
        EntityCountDiagnosticsPlugin::default(),
        SystemInformationDiagnosticsPlugin,
    ))
    .add_plugins((DiagnosticsOverlayPlugin, FeathersPlugins))
    .insert_resource(UiTheme(create_dark_theme()))
    .add_systems(Startup, (setup, ignore_overlay_plane_picking))
    .add_systems(
        Update,
        (
            toggle_debug.run_if(input_just_pressed(KeyCode::KeyD)),
            toggle_debug_ui.run_if(input_just_pressed(KeyCode::KeyU)),
            close_on_esc.in_set(AppSystems::RecordInput),
            log_transitions::<Screen>,
            log_transitions::<Phase>,
        ),
    );
}

fn setup(mut commands: Commands) {
    commands.spawn_scene(debug_ui());
}

/// `DiagnosticsOverlayPlugin` spawns a full-window, hoverable (but non-blocking) plane on top of
/// everything. Every click therefore also "presses" the plane, which makes bevy's click-to-focus
/// clear the input focus, breaking widgets that rely on keeping focus (e.g. feathers number inputs
/// drop out of scrubbing mode). The overlays themselves are separate pickable entities, so the plane
/// doesn't need to be pickable at all.
fn ignore_overlay_plane_picking(mut planes: Query<&mut Pickable, With<DiagnosticsOverlayPlane>>) {
    for mut pickable in &mut planes {
        *pickable = Pickable::IGNORE;
    }
}

fn toggle_debug(
    mut commands: Commands,
    mut debug_info: ResMut<DebugState>,
    overlays: Query<Entity, With<DiagnosticsOverlay>>,
) {
    debug_info.diagnostics_enabled = !debug_info.diagnostics_enabled;

    if debug_info.diagnostics_enabled {
        commands.spawn((DiagnosticsOverlay::fps(),));
    } else {
        for o in overlays {
            commands.entity(o).despawn();
        }
    }
}

fn toggle_debug_ui(
    mut debug_state: ResMut<DebugState>,
    mut query: Query<&mut Visibility, With<DebugUi>>,
) -> Result<(), BevyError> {
    let mut vis = query.single_mut()?;
    if debug_state.debug_ui_enabled {
        *vis = Visibility::Hidden;
        debug_state.debug_ui_enabled = false;
    } else {
        *vis = Visibility::Inherited;
        debug_state.debug_ui_enabled = true;
    }

    Ok(())
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

/// Marker component for the debug UI.
#[derive(Component, Default, Clone, Copy)]
pub struct DebugUi;

fn debug_ui() -> impl Scene {
    bsn! {
        DebugUi
        Visibility::Hidden
        Node {
            position_type: PositionType::Absolute,
            top: px(10.0),
            bottom: px(10.0),
            right: px(10.0),
            width: vh(33.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Start,
            column_gap: px(8.0),
        }
        TabGroup
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            @pane()
            Node {
                width: percent(100.0),
                height: percent(100.0),
            }
            Children [
                @pane_header() Children [
                    @label("Debug UI blablablablablabla")
                ]
                --
                @pane_body() Children [
                    @subpane() Children [
                        @subpane_header() Children [
                            @caption("Chromatic Aberration")
                        ]
                        --
                        @subpane_body() Children [
                            @label_small("intensity")
                            --
                            @FeathersNumberInput
                            NumberInputPrecision(2)
                            NumberInputStep(0.1)
                            HardLimit::f32(0.0..=1.0)
                            NumberInputValue::F32(0.02)
                            on(|value_change: On<ValueChange<f32>>, mut chromatic_aberration: Query<&mut ChromaticAberration>, mut commands: Commands| {
                                commands.entity(value_change.source).insert(NumberInputValue::F32(value_change.value));
                                if value_change.is_final {
                                    chromatic_aberration.single_mut().unwrap().intensity = value_change.value;
                                }
                            })
                            --
                            @label_small("max samples")
                            --
                            @FeathersNumberInput
                            NumberInputPrecision(0)
                            NumberInputStep(1.0)
                            HardLimit::i32(0..=32)
                            NumberInputValue::I32(8)
                            on(|value_change: On<ValueChange<i32>>, mut chromatic_aberration: Query<&mut ChromaticAberration>, mut commands: Commands| {
                                commands.entity(value_change.source).insert(NumberInputValue::I32(value_change.value));
                                if value_change.is_final {
                                    chromatic_aberration.single_mut().unwrap().max_samples = value_change.value as u32;
                                }
                            })
                        ]
                    ]
                    --
                    @subpane() Children [
                        @subpane_header() Children [
                            @caption("Lens distortion")
                        ]
                        --
                        @subpane_body() Children [
                            @label_small("intensity")
                            --
                            @FeathersNumberInput
                            NumberInputPrecision(2)
                            NumberInputStep(0.1)
                            NumberInputValue::F32(0.5)
                            on(|value_change: On<ValueChange<f32>>, mut lens_distortion: Query<&mut LensDistortion>, mut commands: Commands| {
                                commands.entity(value_change.source).insert(NumberInputValue::F32(value_change.value));
                                if value_change.is_final {
                                    lens_distortion.single_mut().unwrap().intensity = value_change.value;
                                }
                            })
                            --
                            @label_small("scale")
                            --
                            @FeathersNumberInput
                            NumberInputPrecision(2)
                            NumberInputStep(0.1)
                            HardLimit::f32(0.0..=10.0)
                            NumberInputValue::F32(1.0)
                            on(|value_change: On<ValueChange<f32>>, mut lens_distortion: Query<&mut LensDistortion>, mut commands: Commands| {
                                commands.entity(value_change.source).insert(NumberInputValue::F32(value_change.value));
                                if value_change.is_final {
                                    lens_distortion.single_mut().unwrap().scale = value_change.value;
                                }
                            })
                            --
                            @label_small("multiplier")
                            --
                            Node {
                                display: Display::Flex,
                                flex_direction: FlexDirection::Row,
                                row_gap: px(8.0),
                            }
                            Children [
                                @FeathersNumberInput {
                                    @label_text: "X",
                                    @sigil_color: tokens::TEXT_INPUT_X_AXIS,
                                }
                                NumberInputPrecision(2)
                                NumberInputStep(0.1)
                                NumberInputValue::F32(1.0)
                                on(|value_change: On<ValueChange<f32>>, mut lens_distortion: Query<&mut LensDistortion>, mut commands: Commands| {
                                    commands.entity(value_change.source).insert(NumberInputValue::F32(value_change.value));
                                    if value_change.is_final {
                                        lens_distortion.single_mut().unwrap().multiplier.x = value_change.value;
                                    }
                                })
                                --
                                @FeathersNumberInput {
                                    @label_text: "Y",
                                    @sigil_color: tokens::TEXT_INPUT_Y_AXIS,
                                }
                                NumberInputPrecision(2)
                                NumberInputStep(0.1)
                                NumberInputValue::F32(1.0)
                                on(|value_change: On<ValueChange<f32>>, mut lens_distortion: Query<&mut LensDistortion>, mut commands: Commands| {
                                    commands.entity(value_change.source).insert(NumberInputValue::F32(value_change.value));
                                    if value_change.is_final {
                                        lens_distortion.single_mut().unwrap().multiplier.y = value_change.value;
                                    }
                                })
                            ]
                        ]
                    ]
                    --
                    @subpane() Children [
                        @subpane_header() Children [
                            @caption("Scanlines")
                        ]
                        --
                        @subpane_body() Children [
                            @label_small("num lines")
                            --
                            @FeathersNumberInput
                            NumberInputPrecision(0)
                            HardLimit::i32(0..=1000)
                            NumberInputStep(10.0)
                            NumberInputValue::I32(200)
                            on(|value_change: On<ValueChange<i32>>, mut scanlines: Query<&mut Scanlines>, mut commands: Commands| {
                                commands.entity(value_change.source).insert(NumberInputValue::I32(value_change.value));
                                if value_change.is_final {
                                    scanlines.single_mut().unwrap().num_lines = value_change.value as u32;
                                }
                            })
                            --
                            @label_small("intensity")
                            --
                            @FeathersNumberInput
                            NumberInputPrecision(2)
                            NumberInputStep(0.1)
                            HardLimit::f32(0.0..=1.0)
                            NumberInputValue::F32(0.5)
                            on(|value_change: On<ValueChange<f32>>, mut scanlines: Query<&mut Scanlines>, mut commands: Commands| {
                                commands.entity(value_change.source).insert(NumberInputValue::F32(value_change.value));
                                if value_change.is_final {
                                    scanlines.single_mut().unwrap().intensity = value_change.value;
                                }
                            })
                        ]
                    ]
                ]
            ]
        ]
    }
}
