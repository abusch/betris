use bevy::prelude::*;
use bevy_enhanced_input::{
    condition::DEFAULT_ACTUATION,
    prelude::{Press, *},
};

pub fn plugin(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin)
        .add_input_context::<InGame>()
        .add_input_condition::<AutoRepeat>()
        .add_observer(toggle_pause);
}

pub fn bindings(commands: &mut Commands) {
    commands.spawn((
        InGame,
        actions!(InGame[
            (Action::<Left>::new(), AutoRepeat::new(0.3, 0.05), bindings![KeyCode::ArrowLeft],),
            (Action::<Right>::new(), AutoRepeat::new(0.3, 0.05), bindings![KeyCode::ArrowRight],),
            (Action::<RotateLeft>::new(), Press::default(), bindings![KeyCode::KeyZ, KeyCode::ArrowUp],),
            (Action::<RotateRight>::new(), Press::default(), bindings![KeyCode::KeyX],),
            (Action::<SoftDrop>::new(), Down::default(), bindings![KeyCode::ArrowDown],),
            (Action::<HardDrop>::new(), Press::default(), bindings![KeyCode::Space],),
            (Action::<Pause>::new(), Press::default(), bindings![KeyCode::KeyP],),
        ]),
    ));
}

#[derive(Debug, Default, Component)]
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
pub struct Pause;

fn toggle_pause(_trigger: On<Start<Pause>>, mut time: ResMut<Time<Virtual>>) {
    if time.is_paused() {
        time.unpause();
    } else {
        time.pause();
    }
}

#[derive(Debug, Component)]
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
            time_kind: TimeKind::Virtual,
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
    ) -> ActionState {
        let previously_actuated = self.actuated;
        self.actuated = value.is_actuated(self.actuation);

        if self.actuated {
            if !previously_actuated {
                // First time fire
                ActionState::Fired
            } else if self.is_repeating {
                self.repeat_timer.tick(time.delta_kind(self.time_kind));
                if self.repeat_timer.just_finished() {
                    ActionState::Fired
                } else {
                    ActionState::Ongoing
                }
            } else {
                self.initial_timer.tick(time.delta_kind(self.time_kind));
                if self.initial_timer.is_finished() {
                    // Start auto-repeat
                    self.is_repeating = true;
                    ActionState::Fired
                } else {
                    ActionState::Ongoing
                }
            }
        } else {
            self.initial_timer.reset();
            self.repeat_timer.reset();
            ActionState::None
        }
    }
}
