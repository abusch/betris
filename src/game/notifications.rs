//! On-screen text telling the player about what they achieved (see section 13.1.5 of the
//! guideline).

use bevy::prelude::*;

use super::{
    SCALE,
    score::{Score, ScoreAction, ScoreNotification},
    ui::game_font,
};
use crate::{AppSystems, screen::Screen};

/// The middle of the visible part of the Matrix
const INSIDE_MATRIX: Vec2 = Vec2::new(0.0, -SCALE);
/// Below the Next Queue
const OUTSIDE_MATRIX: Vec2 = Vec2::new(144.0, -SCALE);
/// The guideline only allows notifications inside the Matrix up to this level, as they would
/// otherwise get in the way at higher fall speeds.
const LAST_LEVEL_INSIDE_MATRIX: u64 = 10;
/// In front of the Matrix and its blocks
const Z: f32 = 10.0;

/// Brighter than white so that the text glows, as otherwise post-processing makes it look dull
const COLOR: Color = Color::LinearRgba(LinearRgba::rgb(4.0, 3.0, 1.0));

/// How long a notification stays on screen, in seconds
const LIFETIME: f32 = 1.2;
/// When the notification starts fading out, as a fraction of its lifetime
const FADE_START: f32 = 0.5;
/// How far a notification moves up over its lifetime
const RISE: f32 = 1.5 * SCALE;
/// How far apart notifications are stacked
const LINE_HEIGHT: f32 = 15.0;
/// How quickly a notification slides out of the way of newer ones
const STACK_DECAY_RATE: f32 = 20.0;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (spawn_notifications, animate_notifications)
            .chain()
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Clone, Component)]
struct Notification {
    origin: Vec2,
    timer: Timer,
    /// The number of newer notifications stacked below this one
    slot: usize,
    /// How far up the stack this notification currently is, sliding towards its slot
    stack_height: f32,
}

impl Notification {
    fn new(origin: Vec2, slot: usize) -> Self {
        Self {
            origin,
            timer: Timer::from_seconds(LIFETIME, TimerMode::Once),
            slot,
            stack_height: slot as f32 * LINE_HEIGHT,
        }
    }

    fn translation(&self) -> Vec3 {
        // Ease out, so that the text quickly pops up then settles
        let rise = RISE * (1.0 - (1.0 - self.timer.fraction()).powi(2));
        (self.origin + Vec2::Y * (self.stack_height + rise)).extend(Z)
    }

    fn alpha(&self) -> f32 {
        let fade = (self.timer.fraction() - FADE_START) / (1.0 - FADE_START);
        1.0 - fade.clamp(0.0, 1.0)
    }
}

fn origin(level: u64) -> Vec2 {
    if level <= LAST_LEVEL_INSIDE_MATRIX {
        INSIDE_MATRIX
    } else {
        OUTSIDE_MATRIX
    }
}

/// Fit the longest text in the width of the Matrix, and keep the actions most prominent.
fn font_size(notification: ScoreNotification) -> f32 {
    match notification {
        ScoreNotification::Action(ScoreAction::MiniTSpinSingle) => 10.0,
        ScoreNotification::Action(_) => 13.0,
        _ => 11.0,
    }
}

/// Stack the new notifications from top to bottom, under any still on screen.
fn spawn_notifications(
    mut commands: Commands,
    mut messages: MessageReader<ScoreNotification>,
    score: Res<Score>,
    mut existing: Query<&mut Notification>,
) {
    let new: Vec<_> = messages.read().copied().collect();
    for mut notification in &mut existing {
        notification.slot += new.len();
    }

    let origin = origin(score.level());
    for (i, message) in new.iter().enumerate() {
        let notification = Notification::new(origin, new.len() - 1 - i);
        let transform = Transform::from_translation(notification.translation());
        let text = message.text();
        let font = font_size(*message);
        commands.spawn_scene(bsn! {
            Name("Notification")
            DespawnOnExit<Screen>(Screen::Gameplay)
            Text2d(text)
            @game_font(font)
            TextColor(COLOR)
            notification
            transform
        });
    }
}

fn animate_notifications(
    mut commands: Commands,
    time: Res<Time>,
    mut notifications: Query<(Entity, &mut Notification, &mut Transform, &mut TextColor)>,
) {
    for (entity, mut notification, mut transform, mut color) in &mut notifications {
        notification.timer.tick(time.delta());
        if notification.timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let target = notification.slot as f32 * LINE_HEIGHT;
        notification
            .stack_height
            .smooth_nudge(&target, STACK_DECAY_RATE, time.delta_secs());
        transform.translation = notification.translation();
        color.0.set_alpha(notification.alpha());
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::{ecs::system::RunSystemOnce, scene::ScenePlugin};

    use super::*;
    use crate::game::score::ScoreEvent;

    fn app(level: u8) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), ScenePlugin))
            .init_asset::<Font>()
            .add_message::<ScoreNotification>();
        let mut score = Score::default();
        score.handle_event(&ScoreEvent::LevelStart(level));
        app.insert_resource(score);
        app
    }

    fn notify(app: &mut App, notifications: &[ScoreNotification]) {
        let world = app.world_mut();
        world.write_message_batch(notifications.iter().copied());
        world.run_system_once(spawn_notifications).unwrap();
        world.flush();
        // Don't see the same messages again next time
        world.resource_mut::<Messages<ScoreNotification>>().clear();
    }

    /// The text and position of the notifications on screen, from top to bottom
    fn on_screen(app: &mut App) -> Vec<(String, Vec2)> {
        let world = app.world_mut();
        let mut notifications: Vec<_> = world
            .query_filtered::<(&Text2d, &Transform), With<Notification>>()
            .iter(world)
            .map(|(text, transform)| (text.0.clone(), transform.translation.truncate()))
            .collect();
        notifications.sort_by(|a, b| b.1.y.total_cmp(&a.1.y));
        notifications
    }

    fn animate(app: &mut App, seconds: f32) {
        let world = app.world_mut();
        world
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(seconds));
        world.run_system_once(animate_notifications).unwrap();
    }

    #[test]
    fn notifications_of_a_lock_down_read_from_top_to_bottom() {
        let mut app = app(1);
        notify(
            &mut app,
            &[
                ScoreNotification::BackToBack,
                ScoreNotification::Action(ScoreAction::Tetris),
                ScoreNotification::LevelUp(2),
            ],
        );

        let texts: Vec<_> = on_screen(&mut app).into_iter().map(|(t, _)| t).collect();
        assert_eq!(texts, ["Back-to-Back", "Tetris", "Level 2"]);
    }

    #[test]
    fn notifications_appear_outside_the_matrix_after_level_10() {
        let tetris = [ScoreNotification::Action(ScoreAction::Tetris)];
        let mut level_10 = app(10);
        notify(&mut level_10, &tetris);
        assert_eq!(on_screen(&mut level_10)[0].1, INSIDE_MATRIX);

        let mut level_11 = app(11);
        notify(&mut level_11, &tetris);
        assert_eq!(on_screen(&mut level_11)[0].1, OUTSIDE_MATRIX);
    }

    #[test]
    fn new_notifications_push_existing_ones_up() {
        let mut app = app(1);
        notify(&mut app, &[ScoreNotification::Action(ScoreAction::Tetris)]);
        notify(&mut app, &[ScoreNotification::Combo(1)]);
        // Long enough to slide into place, but not to disappear
        for _ in 0..10 {
            animate(&mut app, 0.05);
        }

        let notifications = on_screen(&mut app);
        assert_eq!(notifications[0].0, "Tetris");
        assert_eq!(notifications[1].0, "Combo x1");
        let gap = notifications[0].1.y - notifications[1].1.y;
        assert!((gap - LINE_HEIGHT).abs() < 0.1, "gap was {gap}");
    }

    #[test]
    fn notifications_rise_and_fade_before_disappearing() {
        let mut app = app(1);
        notify(&mut app, &[ScoreNotification::Action(ScoreAction::Tetris)]);
        let alpha = |app: &mut App| {
            let world = app.world_mut();
            world
                .query_filtered::<&TextColor, With<Notification>>()
                .single(world)
                .unwrap()
                .0
                .alpha()
        };
        assert_eq!(alpha(&mut app), 1.0);

        animate(&mut app, LIFETIME * FADE_START);
        let halfway = on_screen(&mut app)[0].1.y;
        assert!(halfway > INSIDE_MATRIX.y);
        assert_eq!(alpha(&mut app), 1.0);

        animate(&mut app, LIFETIME * (1.0 - FADE_START) / 2.0);
        assert!(on_screen(&mut app)[0].1.y > halfway);
        assert!((alpha(&mut app) - 0.5).abs() < 0.01);

        animate(&mut app, LIFETIME);
        assert!(on_screen(&mut app).is_empty());
    }
}
