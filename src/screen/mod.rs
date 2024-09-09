use bevy::prelude::*;

mod gameplay;
mod splash;

pub fn plugin(app: &mut App) {
    app.init_state::<Screen>()
        .enable_state_scoped_entities::<Screen>()
        .add_plugins((splash::plugin, gameplay::plugin));

    // Skip the splash screen in dev mode and go straight to the playing screen
    #[cfg(feature = "dev")]
    app.insert_state(Screen::Gameplay);
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Screen {
    #[default]
    Splash,
    Gameplay,
}
