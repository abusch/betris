use bevy::prelude::*;

mod camera;
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

        app.add_plugins((DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Betris".into(),
                    resolution: camera::WINDOW_SIZE.into(),
                    resizable: false,
                    ..default()
                }),
                ..default()
            })
            // Pixel art
            .set(ImagePlugin::default_nearest()),))
            .insert_resource(ClearColor(Srgba::rgb(0.01, 0.01, 0.03).into()));

        app.add_plugins((
            camera::plugin,
            game::plugin,
            screen::plugin,
            material::scanlines::plugin,
        ));

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
