use bevy::prelude::*;

mod dev_tools;
mod game;
mod pieces;
mod pos;
mod screen;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Order new `AppStep` variants by adding them here:
        app.configure_sets(
            Update,
            (AppSet::TickTimers, AppSet::RecordInput, AppSet::Update).chain(),
        );

        app.add_plugins(DefaultPlugins)
            .insert_resource(ClearColor(Color::BLACK))
            .add_systems(Startup, setup)
            .add_plugins((game::plugin, screen::plugin));

        // TODO: disable in release mode
        app.add_plugins(dev_tools::plugin);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum AppSet {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
