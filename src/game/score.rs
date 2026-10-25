use bevy::prelude::*;

use crate::{AppSystems, screen::Screen};

pub fn plugin(app: &mut App) {
    app.add_message::<ScoreEvent>()
        .add_systems(OnEnter(Screen::Gameplay), setup)
        .add_systems(
            Update,
            update
                .run_if(in_state(Screen::Gameplay))
                .in_set(AppSystems::Update),
        )
        .add_systems(OnExit(Screen::Gameplay), cleanup);
}

#[derive(Resource, Reflect)]
pub struct Score {
    level: u64,
    score: u64,
    lines_cleared: u64,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            level: 1,
            score: 0,
            lines_cleared: 0,
        }
    }
}

impl Score {
    /// Number of lines to clear on each level (the guideline's fixed goal system)
    const LINES_PER_LEVEL: u64 = 10;
    const MAX_LEVEL: u64 = 15;

    /// Add the given number of points applying the level factor
    pub fn add_with_mult(&mut self, points: u64) {
        self.score += points * self.level;
    }

    /// Count cleared lines and level up once the goal is reached.
    ///
    /// With the fixed goal system, starting on level n requires clearing 10 x n lines to reach the
    /// next level, then 10 lines per level, so the goal only depends on the total number of lines.
    fn add_lines(&mut self, lines: u64) {
        self.lines_cleared += lines;
        if self.lines_cleared >= Self::LINES_PER_LEVEL * self.level && self.level < Self::MAX_LEVEL
        {
            self.level += 1;
            info!("Level up! Now on level {}", self.level);
        }
    }

    pub fn handle_event(&mut self, event: &ScoreEvent) {
        match event {
            ScoreEvent::LevelStart(level) => {
                self.level = *level as u64;
                self.score = 0;
                self.lines_cleared = 0;
            }
            ScoreEvent::Single => {
                self.add_with_mult(100);
                self.add_lines(1);
            }
            ScoreEvent::Double => {
                self.add_with_mult(300);
                self.add_lines(2);
            }
            ScoreEvent::Triple => {
                self.add_with_mult(500);
                self.add_lines(3);
            }
            ScoreEvent::Tetris => {
                self.add_with_mult(800);
                self.add_lines(4);
            }
            ScoreEvent::MiniTSpin => self.add_with_mult(100),
            ScoreEvent::MiniTSpinSingle => self.add_with_mult(200),
            ScoreEvent::TSpin => self.add_with_mult(400),
            ScoreEvent::TSpinSingle => self.add_with_mult(800),
            ScoreEvent::TSpinDouble => self.add_with_mult(1200),
            ScoreEvent::TSpinTriple => self.add_with_mult(1600),
            ScoreEvent::SoftDrop(n) => self.score += *n as u64,
            ScoreEvent::HardDrop(n) => self.score += *n as u64 * 2,
        }
    }

    pub fn formatted(&self) -> String {
        format!("{:06}", self.score)
    }

    pub fn level(&self) -> u64 {
        self.level
    }

    pub fn lines_cleared(&self) -> u64 {
        self.lines_cleared
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Message)]
#[allow(dead_code)]
pub enum ScoreEvent {
    LevelStart(u8),
    Single,
    Double,
    Triple,
    Tetris,
    MiniTSpin,
    MiniTSpinSingle,
    TSpin,
    TSpinSingle,
    TSpinDouble,
    TSpinTriple,
    SoftDrop(u8),
    HardDrop(u8),
}

fn setup(mut commands: Commands) {
    commands.init_resource::<Score>();
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Score>();
}

fn update(mut score: ResMut<Score>, mut events: MessageReader<ScoreEvent>) {
    for event in events.read() {
        score.handle_event(event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score_at_level(level: u8) -> Score {
        let mut score = Score::default();
        score.handle_event(&ScoreEvent::LevelStart(level));
        score
    }

    #[test]
    fn clearing_10_lines_levels_up() {
        let mut score = score_at_level(1);
        for _ in 0..2 {
            score.handle_event(&ScoreEvent::Tetris);
        }
        assert_eq!(score.level(), 1);

        score.handle_event(&ScoreEvent::Double);
        assert_eq!(score.level(), 2);

        // The next level up is 10 lines later
        for _ in 0..9 {
            score.handle_event(&ScoreEvent::Single);
        }
        assert_eq!(score.level(), 2);
        score.handle_event(&ScoreEvent::Single);
        assert_eq!(score.level(), 3);
    }

    #[test]
    fn line_clear_that_levels_up_scores_at_previous_level() {
        let mut score = score_at_level(1);
        score.handle_event(&ScoreEvent::Tetris);
        score.handle_event(&ScoreEvent::Tetris);
        score.handle_event(&ScoreEvent::Double);
        assert_eq!(score.level(), 2);
        assert_eq!(score.score, 800 + 800 + 300);
    }

    #[test]
    fn starting_on_a_higher_level_requires_the_lines_of_previous_levels() {
        // Starting on level 4 takes 40 lines to reach level 5
        let mut score = score_at_level(4);
        for _ in 0..9 {
            score.handle_event(&ScoreEvent::Tetris);
        }
        assert_eq!(score.level(), 4);
        score.handle_event(&ScoreEvent::Tetris);
        assert_eq!(score.level(), 5);
    }

    #[test]
    fn level_stops_at_15() {
        let mut score = score_at_level(15);
        for _ in 0..100 {
            score.handle_event(&ScoreEvent::Tetris);
        }
        assert_eq!(score.level(), 15);
    }
}
