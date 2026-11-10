//! Pausing the game: game time stops, and the gameplay actions are ignored.

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use super::input::{InGame, Pause};
use crate::screen::Screen;

pub fn plugin(app: &mut App) {
    app.add_sub_state::<IsPaused>()
        .add_systems(OnEnter(IsPaused::Paused), pause)
        .add_systems(OnExit(IsPaused::Paused), resume)
        .add_observer(toggle_pause);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, SubStates, Reflect)]
#[source(Screen = Screen::Gameplay)]
pub enum IsPaused {
    #[default]
    Running,
    Paused,
}

fn toggle_pause(
    _trigger: On<Start<Pause>>,
    state: Option<Res<State<IsPaused>>>,
    mut next_state: ResMut<NextState<IsPaused>>,
) {
    let Some(state) = state else {
        return;
    };
    next_state.set(match **state {
        IsPaused::Running => IsPaused::Paused,
        IsPaused::Paused => IsPaused::Running,
    });
}

fn pause(
    mut commands: Commands,
    mut time: ResMut<Time<Virtual>>,
    contexts: Query<Entity, With<InGame>>,
) {
    time.pause();
    for context in &contexts {
        commands
            .entity(context)
            .insert(ContextActivity::<InGame>::INACTIVE);
    }
}

/// This also runs when leaving the Gameplay screen while paused.
fn resume(
    mut commands: Commands,
    mut time: ResMut<Time<Virtual>>,
    contexts: Query<Entity, With<InGame>>,
) {
    time.unpause();
    for context in &contexts {
        commands
            .entity(context)
            .insert(ContextActivity::<InGame>::ACTIVE);
    }
}

#[cfg(test)]
mod tests {
    use bevy::{input::InputPlugin, scene::ScenePlugin, state::app::StatesPlugin};

    use super::*;
    use crate::game::input::{self, HardDrop};

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ScenePlugin,
            StatesPlugin,
            InputPlugin,
        ))
        .insert_state(Screen::Gameplay)
        .add_plugins((input::plugin, plugin));
        app.finish();
        input::bindings(&mut app.world_mut().commands());
        app.update();
        app
    }

    /// Press `key` for one update, then release it for one update
    fn tap(app: &mut App, key: KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(key);
        app.update();
    }

    /// Whether pressing Space hard drops
    fn hard_drops(app: &mut App) -> bool {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        app.update();
        let world = app.world_mut();
        let fired = world
            .query_filtered::<&ActionEvents, With<Action<HardDrop>>>()
            .single(world)
            .unwrap()
            .contains(ActionEvents::FIRE);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::Space);
        app.update();
        fired
    }

    fn is_paused(app: &App) -> bool {
        let paused = *app.world().resource::<State<IsPaused>>().get() == IsPaused::Paused;
        assert_eq!(paused, app.world().resource::<Time<Virtual>>().is_paused());
        paused
    }

    #[test]
    fn gameplay_actions_are_ignored_while_paused() {
        let mut app = app();
        assert!(hard_drops(&mut app));

        tap(&mut app, KeyCode::KeyP);
        assert!(is_paused(&app));
        assert!(!hard_drops(&mut app));

        tap(&mut app, KeyCode::KeyP);
        assert!(!is_paused(&app));
        assert!(hard_drops(&mut app));
    }

    #[test]
    fn holding_the_pause_button_does_not_unpause() {
        let mut app = app();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyP);
        for _ in 0..10 {
            app.update();
        }
        assert!(is_paused(&app));
    }

    #[test]
    fn leaving_the_gameplay_screen_unpauses_time() {
        let mut app = app();
        tap(&mut app, KeyCode::KeyP);
        assert!(is_paused(&app));

        app.world_mut()
            .resource_mut::<NextState<Screen>>()
            .set(Screen::Splash);
        app.update();
        assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    }
}
