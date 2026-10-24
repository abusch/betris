use bevy::prelude::*;

use crate::{AppSystems, game::ui::game_font};

use super::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Splash), enter_splash)
        .add_systems(OnExit(Screen::Splash), exit_splash)
        .add_systems(
            Update,
            countdown
                .in_set(AppSystems::TickTimers)
                .run_if(in_state(Screen::Splash)),
        );
}

#[derive(Resource, Deref, DerefMut)]
struct SplashTime(Timer);

fn enter_splash(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        DespawnOnExit<Screen>(Screen::Splash)
        Text2d("Betris")
        @game_font(78.0)
        TextColor(Color::WHITE)
        TextLayout { justify: Justify::Center }
    });

    commands.insert_resource(SplashTime(Timer::from_seconds(2.0, TimerMode::Once)));
}

fn exit_splash(mut commands: Commands) {
    commands.remove_resource::<SplashTime>();
}

fn countdown(mut next: ResMut<NextState<Screen>>, time: Res<Time>, mut timer: ResMut<SplashTime>) {
    if timer.tick(time.delta()).is_finished() {
        next.set(Screen::Gameplay);
    }
}
