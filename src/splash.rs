use bevy::prelude::*;

use crate::AppState;

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Splash), splash_setup)
            .add_systems(Update, countdown.run_if(in_state(AppState::Splash)));
    }
}

#[derive(Resource, Deref, DerefMut)]
struct SplashTime(Timer);

fn splash_setup(mut commands: Commands, assets: Res<AssetServer>) {
    let font_handle: Handle<Font> = assets.load("fonts/BungeeSpice-Regular.ttf");

    commands
        .spawn(Text2dBundle {
            text: Text::from_section(
                "Betris",
                TextStyle {
                    font: font_handle,
                    font_size: 78.0,
                    color: Color::WHITE,
                },
            )
            .with_justify(JustifyText::Center),
            ..default()
        })
        .insert(StateScoped(AppState::Splash));

    commands.insert_resource(SplashTime(Timer::from_seconds(2.0, TimerMode::Once)));
}

fn countdown(
    mut next: ResMut<NextState<AppState>>,
    time: Res<Time>,
    mut timer: ResMut<SplashTime>,
) {
    if timer.tick(time.delta()).finished() {
        next.set(AppState::InGame);
    }
}
