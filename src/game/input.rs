use bevy::prelude::*;
use bevy_enhanced_input::{
    condition::DEFAULT_ACTUATION,
    prelude::{Press, *},
};

use crate::screen::Screen;

pub fn plugin(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin)
        .add_input_context::<InGame>()
        .add_input_condition::<AutoRepeat>()
        .add_observer(toggle_pause);
}

pub fn bindings(commands: &mut Commands) {
    commands.spawn_scene(bsn! {
        InGame
        DespawnOnExit<Screen>(Screen::Gameplay)
        Actions<InGame> [
            Action::<Left>::new()
            ~AutoRepeat::new(0.3, 0.05)
            Bindings [
                Binding(KeyCode::ArrowLeft)
                --
                Binding(GamepadButton::DPadLeft)
            ]
            --
            Action::<Right>::new()
            ~AutoRepeat::new(0.3, 0.05)
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

#[derive(Debug, Clone, Component)]
pub struct AutoRepeat {
    actuated: bool,
    actuation: f32,
    time_kind: TimeKind,
    initial_timer: Timer,
    repeat_timer: Timer,
    is_repeating: bool,
}

impl AutoRepeat {
    pub fn new(initial_delay: f32, interval: f32) -> Self {
        Self {
            actuated: false,
            actuation: DEFAULT_ACTUATION,
            time_kind: TimeKind::Auto,
            initial_timer: Timer::from_seconds(initial_delay, TimerMode::Once),
            repeat_timer: Timer::from_seconds(interval, TimerMode::Repeating),
            is_repeating: false,
        }
    }
}

impl InputCondition for AutoRepeat {
    fn evaluate(
        &mut self,
        _actions: &ActionsQuery,
        time: &ContextTime,
        value: ActionValue,
    ) -> TriggerState {
        let previously_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if self.actuated {
            if !previously_actuated {
                // First time fire
                TriggerState::Fired
            } else if self.is_repeating {
                self.repeat_timer.tick(time.delta_kind(self.time_kind));
                if self.repeat_timer.just_finished() {
                    TriggerState::Fired
                } else {
                    TriggerState::Ongoing
                }
            } else {
                self.initial_timer.tick(time.delta_kind(self.time_kind));
                if self.initial_timer.just_finished() {
                    // Start auto-repeat
                    self.is_repeating = true;
                    TriggerState::Fired
                } else {
                    TriggerState::Ongoing
                }
            }
        } else {
            self.initial_timer.reset();
            self.repeat_timer.reset();
            self.is_repeating = false;
            TriggerState::None
        }
    }
}
