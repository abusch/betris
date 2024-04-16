use bevy::prelude::*;

use crate::AppState;

pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Splash), splash_setup)
            .add_systems(Update, countdown.run_if(in_state(AppState::Splash)))
            .add_systems(OnExit(AppState::Splash), splash_cleanup);
    }
}

#[derive(Resource, Deref, DerefMut)]
struct SplashTime(Timer);

#[derive(Component)]
struct OnSplashScreen;

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
        .insert(OnSplashScreen);

    commands.insert_resource(SplashTime(Timer::from_seconds(2.0, TimerMode::Once)));
}

fn splash_cleanup(mut commands: Commands, query: Query<Entity, With<OnSplashScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive()
    }
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
