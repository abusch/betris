use bevy::prelude::*;

use crate::AppSet;

use super::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Splash), enter_splash)
        .add_systems(OnExit(Screen::Splash), exit_splash)
        .add_systems(
            Update,
            countdown
                .in_set(AppSet::TickTimers)
                .run_if(in_state(Screen::Splash)),
        );
}

#[derive(Resource, Deref, DerefMut)]
struct SplashTime(Timer);

fn enter_splash(mut commands: Commands, assets: Res<AssetServer>) {
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
        .insert(StateScoped(Screen::Splash));

    commands.insert_resource(SplashTime(Timer::from_seconds(2.0, TimerMode::Once)));
}

fn exit_splash(mut commands: Commands) {
    commands.remove_resource::<SplashTime>();
}

fn countdown(mut next: ResMut<NextState<Screen>>, time: Res<Time>, mut timer: ResMut<SplashTime>) {
    if timer.tick(time.delta()).finished() {
        next.set(Screen::Playing);
    }
}
