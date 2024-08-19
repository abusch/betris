use bevy::prelude::*;

mod playing;
mod splash;

pub fn plugin(app: &mut App) {
    app.init_state::<Screen>()
        .enable_state_scoped_entities::<Screen>()
        .add_plugins((splash::plugin, playing::plugin));

    // Skip the splash screen in dev mode and go straight to the playing screen
    #[cfg(feature = "dev")]
    app.insert_state(Screen::Playing);
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Screen {
    #[default]
    Splash,
    Playing,
}
