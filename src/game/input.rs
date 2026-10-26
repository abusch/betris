use std::time::Duration;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::{Press, *};

use crate::screen::Screen;

pub fn plugin(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin)
        .add_input_context::<InGame>()
        .add_observer(toggle_pause);
}

pub fn bindings(commands: &mut Commands) {
    commands.insert_resource(AutoRepeat::default());
    commands.spawn_scene(bsn! {
        InGame
        DespawnOnExit<Screen>(Screen::Gameplay)
        Actions<InGame> [
            Action::<Left>::new()
            ~Down::default()
            Bindings [
                Binding(KeyCode::ArrowLeft)
                --
                Binding(GamepadButton::DPadLeft)
            ]
            --
            Action::<Right>::new()
            ~Down::default()
            Bindings [
                Binding(KeyCode::ArrowRight)
                --
                Binding(GamepadButton::DPadRight)
            ]
            --
            Action::<RotateLeft>::new()
            ~Press::default()
            Bindings [
                Binding(KeyCode::KeyZ)
                --
                Binding(GamepadButton::South)
            ]
            --
            Action::<RotateRight>::new()
            ~Press::default()
            Bindings [
                Binding(KeyCode::KeyX)
                --
                Binding(GamepadButton::East)
            ]
            --
            Action::<SoftDrop>::new()
            ~Down::default()
            Bindings [
                Binding(KeyCode::ArrowDown)
                --
                Binding(GamepadButton::DPadDown)
            ]
            --
            Action::<HardDrop>::new()
            ~Press::default()
            Bindings [
                Binding(KeyCode::Space)
                --
                Binding(GamepadButton::DPadUp)
            ]
            --
            Action::<Hold>::new()
            ~Press::default()
            Bindings [
                Binding(KeyCode::ShiftLeft)
                --
                Binding(KeyCode::KeyC)
                --
                Binding(GamepadButton::LeftTrigger)
                --
                Binding(GamepadButton::RightTrigger)
            ]
            --
            Action::<Pause>::new()
            ~Press::default()
            Bindings [
                Binding(KeyCode::KeyP)
                --
                Binding(GamepadButton::Select)
            ]
            --
            Action::<Restart>::new()
            ~Press::default()
            Bindings [
                Binding(KeyCode::Enter)
                --
                Binding(GamepadButton::Start)
            ]
        ]
    });
}

#[derive(Debug, Default, Clone, Component)]
pub struct InGame;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct Left;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct Right;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct RotateLeft;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct RotateRight;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct SoftDrop;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct HardDrop;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct Hold;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct Pause;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct Restart;

fn toggle_pause(_trigger: On<Start<Pause>>, mut time: ResMut<Time<Virtual>>) {
    if time.is_paused() {
        time.unpause();
    } else {
        time.pause();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

/// State of a move button during the current frame
#[derive(Debug, Default, Clone, Copy)]
struct MoveButton {
    just_pressed: bool,
    held: bool,
}

impl From<&ActionEvents> for MoveButton {
    fn from(events: &ActionEvents) -> Self {
        Self {
            just_pressed: events.contains(ActionEvents::START),
            held: events.contains(ActionEvents::FIRE),
        }
    }
}

/// Left/Right movement with Auto-Repeat (see section 5.2 of the guideline).
///
/// Both directions share the same Auto-Repeat: pressing a move button moves the piece once, and
/// Auto-Repeat kicks in if it is held past the delay. Pressing the opposite button takes over, with
/// the delay applied again, and releasing it goes back to the direction still held, also with the
/// delay applied again.
#[derive(Debug, Resource)]
pub struct AutoRepeat {
    /// The direction being moved in, i.e. the most recently pressed move button still held
    direction: Option<Direction>,
    delay: Timer,
    interval: Timer,
    /// The direction to move the piece in this frame
    movement: Option<Direction>,
}

impl AutoRepeat {
    /// The guideline asks for roughly 0.3s, but that feels sluggish compared to modern games
    const DELAY: Duration = Duration::from_millis(170);
    const INTERVAL: Duration = Duration::from_millis(33);

    /// The direction to move the piece in this frame, if any
    pub fn movement(&self) -> Option<Direction> {
        self.movement
    }

    fn update(
        &mut self,
        left: MoveButton,
        right: MoveButton,
        delta: Duration,
    ) -> Option<Direction> {
        let held = |direction| match direction {
            Direction::Left => left.held,
            Direction::Right => right.held,
        };

        if left.just_pressed {
            return Some(self.start(Direction::Left));
        }
        if right.just_pressed {
            return Some(self.start(Direction::Right));
        }

        let direction = self.direction?;
        if !held(direction) {
            let other = direction.opposite();
            if held(other) {
                return Some(self.start(other));
            }
            self.direction = None;
            return None;
        }

        if self.delay.is_finished() {
            self.interval.tick(delta);
            self.interval.just_finished().then_some(direction)
        } else {
            self.delay.tick(delta);
            self.delay.just_finished().then_some(direction)
        }
    }

    /// Move in `direction`, and start Auto-Repeat after the delay.
    fn start(&mut self, direction: Direction) -> Direction {
        self.direction = Some(direction);
        self.delay.reset();
        self.interval.reset();
        direction
    }
}

impl Default for AutoRepeat {
    fn default() -> Self {
        Self {
            direction: None,
            delay: Timer::new(Self::DELAY, TimerMode::Once),
            interval: Timer::new(Self::INTERVAL, TimerMode::Repeating),
            movement: None,
        }
    }
}

/// This runs during every phase, so that Auto-Repeat carries over to the next piece.
pub fn auto_repeat(
    mut auto_repeat: ResMut<AutoRepeat>,
    left: Single<&ActionEvents, With<Action<Left>>>,
    right: Single<&ActionEvents, With<Action<Right>>>,
    time: Res<Time>,
) {
    auto_repeat.movement = auto_repeat.update((*left).into(), (*right).into(), time.delta());
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRESSED: MoveButton = MoveButton {
        just_pressed: true,
        held: true,
    };
    const HELD: MoveButton = MoveButton {
        just_pressed: false,
        held: true,
    };
    const RELEASED: MoveButton = MoveButton {
        just_pressed: false,
        held: false,
    };
    const DELAY_MS: u64 = AutoRepeat::DELAY.as_millis() as u64;
    const INTERVAL_MS: u64 = AutoRepeat::INTERVAL.as_millis() as u64;

    /// Tick `auto_repeat` in 1ms frames for `millis`, and return the moves made
    fn run(
        auto_repeat: &mut AutoRepeat,
        left: MoveButton,
        right: MoveButton,
        millis: u64,
    ) -> Vec<Direction> {
        (0..millis)
            .filter_map(|_| auto_repeat.update(left, right, Duration::from_millis(1)))
            .collect()
    }

    #[test]
    fn tapping_moves_once() {
        let mut auto_repeat = AutoRepeat::default();
        let frame = Duration::from_millis(10);
        assert_eq!(
            auto_repeat.update(PRESSED, RELEASED, frame),
            Some(Direction::Left)
        );
        assert_eq!(auto_repeat.update(RELEASED, RELEASED, frame), None);
        assert_eq!(run(&mut auto_repeat, RELEASED, RELEASED, 1000), []);
    }

    #[test]
    fn holding_repeats_after_the_delay() {
        let mut auto_repeat = AutoRepeat::default();
        auto_repeat.update(PRESSED, RELEASED, Duration::from_millis(10));
        assert_eq!(run(&mut auto_repeat, HELD, RELEASED, DELAY_MS - 1), []);
        assert_eq!(run(&mut auto_repeat, HELD, RELEASED, 1), [Direction::Left]);
        assert_eq!(
            run(&mut auto_repeat, HELD, RELEASED, 3 * INTERVAL_MS),
            [Direction::Left; 3]
        );
    }

    #[test]
    fn pressing_the_opposite_direction_takes_over_with_the_delay() {
        let mut auto_repeat = AutoRepeat::default();
        auto_repeat.update(PRESSED, RELEASED, Duration::from_millis(10));
        run(&mut auto_repeat, HELD, RELEASED, 500);

        assert_eq!(
            auto_repeat.update(HELD, PRESSED, Duration::from_millis(10)),
            Some(Direction::Right)
        );
        assert_eq!(run(&mut auto_repeat, HELD, HELD, DELAY_MS - 1), []);
        assert_eq!(run(&mut auto_repeat, HELD, HELD, 1), [Direction::Right]);
    }

    #[test]
    fn releasing_goes_back_to_the_direction_still_held_with_the_delay() {
        let mut auto_repeat = AutoRepeat::default();
        auto_repeat.update(PRESSED, RELEASED, Duration::from_millis(10));
        auto_repeat.update(HELD, PRESSED, Duration::from_millis(10));
        run(&mut auto_repeat, HELD, HELD, 500);

        assert_eq!(
            auto_repeat.update(HELD, RELEASED, Duration::from_millis(10)),
            Some(Direction::Left)
        );
        assert_eq!(run(&mut auto_repeat, HELD, RELEASED, DELAY_MS - 1), []);
        assert_eq!(run(&mut auto_repeat, HELD, RELEASED, 1), [Direction::Left]);
    }

    #[test]
    fn releasing_an_older_button_changes_nothing() {
        let mut auto_repeat = AutoRepeat::default();
        auto_repeat.update(PRESSED, RELEASED, Duration::from_millis(10));
        auto_repeat.update(HELD, PRESSED, Duration::from_millis(10));
        run(&mut auto_repeat, HELD, HELD, 100);

        assert_eq!(run(&mut auto_repeat, RELEASED, HELD, DELAY_MS - 101), []);
        assert_eq!(run(&mut auto_repeat, RELEASED, HELD, 1), [Direction::Right]);
    }
}
