use bevy::{
    camera::Hdr,
    core_pipeline::tonemapping::Tonemapping,
    post_process::{
        bloom::Bloom,
        effect_stack::{ChromaticAberration, LensDistortion},
    },
    prelude::*,
    render::view::{ColorGrading, ColorGradingGlobal},
};

use crate::material::scanlines::Scanlines;

#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod material;
mod model;
mod screen;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Order new `AppStep` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        app.add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Betris".into(),
                resolution: (1080, 720).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }),))
            .insert_resource(ClearColor(Color::BLACK))
            .add_systems(Startup, setup);

        app.add_plugins((game::plugin, screen::plugin, material::scanlines::plugin));

        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Hdr,
        ColorGrading {
            global: ColorGradingGlobal {
                exposure: 1.2,
                ..default()
            },
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom::OLD_SCHOOL,
        ChromaticAberration {
            intensity: 0.02,
            ..default()
        },
        LensDistortion {
            intensity: 0.1,
            ..default()
        },
        Scanlines::default(),
    ));
}
