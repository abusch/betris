use std::time::Duration;

use bevy::prelude::*;

#[derive(Default, Resource)]
pub struct Timers {
    pub fall: FallTimer,
    pub lock: LockTimer,
}

impl Timers {
    pub fn tick(&mut self, delta: Duration) {
        self.fall.tick(delta);
        self.lock.tick(delta);
    }
}

#[derive(Deref, DerefMut)]
pub(super) struct FallTimer {
    #[deref]
    timer: Timer,
    normal_duration: Duration,
    softdrop_duration: Duration,
}

impl FallTimer {
    pub fn new() -> Self {
        // TODO make it depend on the current game level
        let normal_duration = Duration::from_millis(1000);
        // Pieces fall down 20x faster during soft drop
        let softdrop_duration = Duration::from_millis(1000 / 20);
        let timer = Timer::new(normal_duration, TimerMode::Repeating);
        Self {
            timer,
            normal_duration,
            softdrop_duration,
        }
    }

    pub fn normal_drop(&mut self) {
        self.timer.set_duration(self.normal_duration);
        self.timer.reset();
    }

    pub fn soft_drop(&mut self) {
        self.timer.set_duration(self.softdrop_duration);
        self.timer.reset();
        self.timer.tick(self.softdrop_duration);
    }
}

impl Default for FallTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deref, DerefMut)]
pub(super) struct LockTimer(Timer);

impl LockTimer {
    pub fn new() -> Self {
        let mut timer = Self(Timer::new(Duration::from_millis(500), TimerMode::Once));
        timer.pause();
        timer
    }
}

impl Default for LockTimer {
    fn default() -> Self {
        Self::new()
    }
}
