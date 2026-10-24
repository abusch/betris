use std::time::Duration;

use bevy::prelude::*;

#[derive(Default, Resource)]
pub struct Timers {
    pub generation: GenerationTimer,
    pub fall: FallTimer,
    pub lock: LockTimer,
}

impl Timers {
    pub fn tick(&mut self, delta: Duration) {
        self.generation.tick(delta);
        self.fall.tick(delta);
        self.lock.tick(delta);
    }
}

/// Delay between a piece locking down and the next one being generated
#[derive(Deref, DerefMut)]
pub(super) struct GenerationTimer(Timer);

impl GenerationTimer {
    pub fn new() -> Self {
        Self(Timer::new(Duration::from_millis(200), TimerMode::Once))
    }
}

impl Default for GenerationTimer {
    fn default() -> Self {
        Self::new()
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

/// Lock down timer, following the guideline's Extended Placement rules.
///
/// The timer only runs while the piece is on a surface. Moving or rotating the piece resets it, but
/// only 15 times, unless the piece falls below the lowest row it has reached so far.
#[derive(Deref, DerefMut)]
pub(super) struct LockTimer {
    #[deref]
    timer: Timer,
    moves_left: u8,
    lowest_y: i8,
}

impl LockTimer {
    const MAX_MOVES: u8 = 15;

    pub fn new() -> Self {
        let mut timer = Timer::new(Duration::from_millis(500), TimerMode::Once);
        timer.pause();
        Self {
            timer,
            moves_left: Self::MAX_MOVES,
            lowest_y: i8::MAX,
        }
    }

    /// Start tracking a new piece at row `y`.
    pub fn start(&mut self, y: i8) {
        self.timer.reset();
        self.timer.pause();
        self.moves_left = Self::MAX_MOVES;
        self.lowest_y = y;
    }

    /// The piece was moved or rotated.
    pub fn on_move(&mut self) {
        if self.moves_left > 0 {
            self.moves_left -= 1;
            self.timer.reset();
        }
    }

    /// The piece is now at row `y`.
    pub fn on_fall(&mut self, y: i8) {
        if y < self.lowest_y {
            self.lowest_y = y;
            self.moves_left = Self::MAX_MOVES;
            self.timer.reset();
        }
    }

    /// Whether the piece has used up all its moves, in which case it locks down as soon as it
    /// touches a surface.
    pub fn moves_exhausted(&self) -> bool {
        self.moves_left == 0
    }
}

impl Default for LockTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A lock timer for a piece at row 10 that has been on a surface for 0.4s
    fn landed_lock_timer() -> LockTimer {
        let mut lock = LockTimer::new();
        lock.start(10);
        lock.unpause();
        lock.tick(Duration::from_millis(400));
        lock
    }

    #[test]
    fn moving_resets_the_timer() {
        let mut lock = landed_lock_timer();
        lock.on_move();
        assert_eq!(lock.elapsed(), Duration::ZERO);
    }

    #[test]
    fn only_15_moves_reset_the_timer() {
        let mut lock = landed_lock_timer();
        for _ in 0..15 {
            assert!(!lock.moves_exhausted());
            lock.on_move();
        }
        assert!(lock.moves_exhausted());

        lock.tick(Duration::from_millis(400));
        lock.on_move();
        assert_eq!(lock.elapsed(), Duration::from_millis(400));
    }

    #[test]
    fn falling_below_lowest_row_gives_moves_back() {
        let mut lock = landed_lock_timer();
        for _ in 0..15 {
            lock.on_move();
        }
        lock.tick(Duration::from_millis(400));

        lock.on_fall(9);
        assert!(!lock.moves_exhausted());
        assert_eq!(lock.elapsed(), Duration::ZERO);
    }

    #[test]
    fn landing_on_a_row_already_reached_changes_nothing() {
        let mut lock = landed_lock_timer();
        lock.on_move();
        lock.tick(Duration::from_millis(400));

        lock.on_fall(10);
        lock.on_fall(11);
        assert_eq!(lock.moves_left, 14);
        assert_eq!(lock.elapsed(), Duration::from_millis(400));
    }
}
