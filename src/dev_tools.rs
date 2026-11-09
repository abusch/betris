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
            FeathersDisclosureToggle, FeathersNumberInput, FeathersScrollbar, HardLimit,
            NumberInputPrecision, NumberInputStep, NumberInputValue, ScrollbarGutter,
        },
        dark_theme::create_dark_theme,
        display::{caption, label, label_small},
        rounded_corners::RoundedCorners,
        theme::{ThemeBackgroundColor, UiTheme},
        tokens,
    },
    input::common_conditions::input_just_pressed,
    input_focus::tab_navigation::TabGroup,
    post_process::effect_stack::{ChromaticAberration, LensDistortion},
    prelude::*,
    ui::Checked,
    ui_widgets::{ControlOrientation, ScrollArea, ValueChange},
};

use crate::{
    AppSystems,
    camera::RESOLUTION,
    game::{Phase, intensities::Intensities},
    material::scanlines::Scanlines,
    screen::Screen,
};

/// Room for the debug UI's scrollbar, in pixels
const SCROLLBAR_GUTTER: f32 = 10.0;

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
                    @label("Debug UI")
                ]
                --
                @pane_body()
                Node {
                    flex_grow: 1.0,
                    min_height: px(0.0),
                }
                Children [
                    // Holds the scrollbar, in the right padding while the sections overflow
                    Node {
                        flex_grow: 1.0,
                        min_height: px(0.0),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Stretch,
                    }
                    ScrollbarGutter(px(SCROLLBAR_GUTTER))
                    Children [
                        #sections
                        Node {
                            flex_grow: 1.0,
                            min_height: px(0.0),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            row_gap: px(4.0),
                            overflow: Overflow::scroll_y(),
                        }
                        ScrollArea
                        Children [
                            @chromatic_aberration_section()
                            --
                            @lens_distortion_section()
                            --
                            @scanlines_section()
                            --
                            @neon_intensity_section()
                        ]
                        --
                        @FeathersScrollbar {
                            @target: #sections,
                            @orientation: {ControlOrientation::Vertical},
                        }
                        Node {
                            position_type: PositionType::Absolute,
                            right: px(0.0),
                            top: px(0.0),
                            bottom: px(0.0),
                            width: px(6.0),
                        }
                    ]
                ]
            ]
        ]
    }
}

/// A subpane titled `title` showing `body`, which can be collapsed by clicking the chevron in its
/// header
fn section(title: &'static str, body: impl SceneList) -> impl Scene {
    bsn! {
        @subpane()
        // Overflow the scroll area rather than squeeze into it
        Node { flex_shrink: 0.0 }
        Children [
            @subpane_header()
            Node { justify_content: JustifyContent::Start }
            Children [
                @FeathersDisclosureToggle
                Checked
                on(toggle_section)
                --
                @caption(title)
            ]
            --
            @subpane_body()
            SectionBody
            Children [ {body} ]
        ]
    }
}

/// Marker component for the body of a collapsible section of the debug UI
#[derive(Component, Default, Clone, Copy)]
struct SectionBody;

/// Expand or collapse the section whose disclosure toggle was clicked. Collapsed sections show
/// their header only, with all of its corners rounded.
fn toggle_section(
    value_change: On<ValueChange<bool>>,
    parents: Query<&ChildOf>,
    children: Query<&Children>,
    mut nodes: Query<&mut Node>,
    bodies: Query<(), With<SectionBody>>,
    mut commands: Commands,
) -> Result {
    let expanded = value_change.value;
    let toggle = value_change.source;
    if expanded {
        commands.entity(toggle).insert(Checked);
    } else {
        commands.entity(toggle).remove::<Checked>();
    }

    let header = parents.get(toggle)?.parent();
    let subpane = parents.get(header)?.parent();
    let mut header_node = nodes.get_mut(header)?;
    header_node.border.bottom = if expanded { px(0.0) } else { px(1.0) };
    header_node.border_radius = if expanded {
        RoundedCorners::Top
    } else {
        RoundedCorners::All
    }
    .to_border_radius(4.0);

    for child in children.get(subpane)?.iter() {
        if bodies.contains(child) {
            nodes.get_mut(child)?.display = if expanded {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
    Ok(())
}

fn chromatic_aberration_section() -> impl Scene {
    bsn! {
        @section("Chromatic Aberration", bsn_list! {
            @label_small("intensity")
            --
            @FeathersNumberInput
            NumberInputPrecision(2)
            NumberInputStep(0.1)
            HardLimit::f32(0.0..=1.0)
            NumberInputValue::F32(0.01)
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
        })
    }
}

fn lens_distortion_section() -> impl Scene {
    bsn! {
        @section("Lens distortion", bsn_list! {
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
        })
    }
}

fn scanlines_section() -> impl Scene {
    bsn! {
        @section("Scanlines", bsn_list! {
            @label_small("num lines")
            --
            @FeathersNumberInput
            NumberInputPrecision(0)
            HardLimit::i32(0..=1000)
            NumberInputStep(10.0)
            NumberInputValue::I32({RESOLUTION.y as i32})
            on(|value_change: On<ValueChange<i32>>, mut scanlines: Query<&mut Scanlines>, mut commands: Commands| {
                commands.entity(value_change.source).insert(NumberInputValue::I32(value_change.value));
                if value_change.is_final {
                    scanlines.single_mut().unwrap().num_lines = value_change.value as u32;
                }
            })
            --
            @label_small("num cols")
            --
            @FeathersNumberInput
            NumberInputPrecision(0)
            HardLimit::i32(0..=1000)
            NumberInputStep(10.0)
            NumberInputValue::I32({RESOLUTION.x as i32})
            on(|value_change: On<ValueChange<i32>>, mut scanlines: Query<&mut Scanlines>, mut commands: Commands| {
                commands.entity(value_change.source).insert(NumberInputValue::I32(value_change.value));
                if value_change.is_final {
                    scanlines.single_mut().unwrap().num_cols = value_change.value as u32;
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
        })
    }
}

fn neon_intensity_section() -> impl Scene {
    bsn! {
        @section("Neon intensity", bsn_list! {
            @label_small("pieces")
            --
            @intensity_input(|intensities: &mut Intensities| &mut intensities.neon)
            --
            @label_small("locked blocks")
            --
            @intensity_input(|intensities: &mut Intensities| &mut intensities.locked)
            --
            @label_small("ghost piece")
            --
            @intensity_input(|intensities: &mut Intensities| &mut intensities.ghost)
            --
            @label_small("matrix frame")
            --
            @intensity_input(|intensities: &mut Intensities| &mut intensities.frame)
        })
    }
}

/// A number input editing the field of `Intensities` that `field` points to
fn intensity_input(field: fn(&mut Intensities) -> &mut f32) -> impl Scene {
    let value = *field(&mut Intensities::default());
    bsn! {
        @FeathersNumberInput
        NumberInputPrecision(2)
        NumberInputStep(0.1)
        HardLimit::f32(0.0..=10.0)
        NumberInputValue::F32(value)
        on(move |value_change: On<ValueChange<f32>>, mut intensities: ResMut<Intensities>, mut commands: Commands| {
            commands.entity(value_change.source).insert(NumberInputValue::F32(value_change.value));
            if value_change.is_final {
                *field(&mut intensities) = value_change.value;
            }
        })
    }
}
