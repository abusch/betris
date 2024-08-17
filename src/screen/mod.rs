use bevy::prelude::*;

mod playing;
mod splash;

pub fn plugin(app: &mut App) {
    app.init_state::<Screen>()
        .enable_state_scoped_entities::<Screen>()
        .add_plugins((splash::plugin, playing::plugin));
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Screen {
    #[default]
    Splash,
    Playing,
}
