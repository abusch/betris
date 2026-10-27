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
    /// Whether a Back-to-Back sequence is in progress, i.e. the last Line Clear was a Tetris or a
    /// T-Spin Line Clear
    back_to_back: bool,
    /// The number of consecutive Line Clears after the first one, or `None` if the last piece
    /// didn't clear any lines
    combo: Option<u64>,
}

impl Default for Score {
    fn default() -> Self {
        Self {
            level: 1,
            score: 0,
            lines_cleared: 0,
            back_to_back: false,
            combo: None,
        }
    }
}

impl Score {
    /// Number of lines to clear on each level (the guideline's fixed goal system)
    const LINES_PER_LEVEL: u64 = 10;
    const MAX_LEVEL: u64 = 15;
    /// Points per combo step, multiplied by the level. The 2009 guideline has no combo bonus, so
    /// this follows later guideline games.
    const COMBO_POINTS: u64 = 50;

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

    /// Score what the player achieved when locking down a piece, including the Back-to-Back and
    /// combo bonuses.
    fn lock_down(&mut self, action: ScoreAction) {
        let lines = action.lines();
        let mut points = action.points() * self.level;

        // T-Spins and Mini T-Spins without a Line Clear neither get the bonus nor break the
        // sequence
        if action.is_back_to_back_action() {
            if self.back_to_back {
                info!("Back-to-Back!");
                points += points / 2;
            }
            self.back_to_back = true;
        } else if lines > 0 {
            self.back_to_back = false;
        }

        if lines > 0 {
            let combo = self.combo.map_or(0, |combo| combo + 1);
            if combo > 0 {
                info!("Combo x{combo}");
            }
            points += Self::COMBO_POINTS * combo * self.level;
            self.combo = Some(combo);
        } else {
            self.combo = None;
        }

        self.score += points;
        // Done last so that a Line Clear that levels up scores at the previous level
        self.add_lines(lines);
    }

    pub fn handle_event(&mut self, event: &ScoreEvent) {
        match event {
            ScoreEvent::LevelStart(level) => {
                *self = Self {
                    level: *level as u64,
                    ..default()
                };
            }
            ScoreEvent::LockDown(action) => self.lock_down(*action),
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
pub enum ScoreEvent {
    LevelStart(u8),
    /// The piece in play locked down
    LockDown(ScoreAction),
    SoftDrop(u8),
    HardDrop(u8),
}

/// A T-Spin, recognised when the T-Tetrimino locks down (see section 9 of the guideline)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Reflect)]
pub enum TSpin {
    Mini,
    Full,
}

/// What the player achieved by locking down a piece (see section 8 of the guideline)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ScoreAction {
    /// A lock down without any Line Clear or T-Spin
    Lock,
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
}

impl ScoreAction {
    /// The action for a piece that cleared `lines` lines, or `None` if that isn't possible.
    pub fn new(lines: usize, t_spin: Option<TSpin>) -> Option<Self> {
        let action = match (t_spin, lines) {
            (None, 0) => Self::Lock,
            (None, 1) => Self::Single,
            (None, 2) => Self::Double,
            (None, 3) => Self::Triple,
            (None, 4) => Self::Tetris,
            (Some(TSpin::Mini), 0) => Self::MiniTSpin,
            (Some(TSpin::Mini), 1) => Self::MiniTSpinSingle,
            // The guideline has no Mini T-Spin Double or Triple, so count those as T-Spins
            (Some(_), 0) => Self::TSpin,
            (Some(_), 1) => Self::TSpinSingle,
            (Some(_), 2) => Self::TSpinDouble,
            (Some(_), 3) => Self::TSpinTriple,
            _ => return None,
        };
        Some(action)
    }

    /// Points awarded before applying the level multiplier
    fn points(self) -> u64 {
        match self {
            Self::Lock => 0,
            Self::Single | Self::MiniTSpin => 100,
            Self::MiniTSpinSingle => 200,
            Self::Double => 300,
            Self::TSpin => 400,
            Self::Triple => 500,
            Self::Tetris | Self::TSpinSingle => 800,
            Self::TSpinDouble => 1200,
            Self::TSpinTriple => 1600,
        }
    }

    fn lines(self) -> u64 {
        match self {
            Self::Lock | Self::MiniTSpin | Self::TSpin => 0,
            Self::Single | Self::MiniTSpinSingle | Self::TSpinSingle => 1,
            Self::Double | Self::TSpinDouble => 2,
            Self::Triple | Self::TSpinTriple => 3,
            Self::Tetris => 4,
        }
    }

    /// Whether this action starts or continues a Back-to-Back sequence
    fn is_back_to_back_action(self) -> bool {
        matches!(
            self,
            Self::Tetris
                | Self::MiniTSpinSingle
                | Self::TSpinSingle
                | Self::TSpinDouble
                | Self::TSpinTriple
        )
    }
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

    /// Lock down pieces achieving each of `actions` in turn
    fn lock_down(score: &mut Score, actions: &[ScoreAction]) {
        for &action in actions {
            score.handle_event(&ScoreEvent::LockDown(action));
        }
    }

    #[test]
    fn clearing_10_lines_levels_up() {
        let mut score = score_at_level(1);
        lock_down(&mut score, &[ScoreAction::Tetris; 2]);
        assert_eq!(score.level(), 1);

        lock_down(&mut score, &[ScoreAction::Double]);
        assert_eq!(score.level(), 2);

        // The next level up is 10 lines later
        lock_down(&mut score, &[ScoreAction::Single; 9]);
        assert_eq!(score.level(), 2);
        lock_down(&mut score, &[ScoreAction::Single]);
        assert_eq!(score.level(), 3);
    }

    #[test]
    fn line_clear_that_levels_up_scores_at_previous_level() {
        let mut score = score_at_level(1);
        use ScoreAction::*;
        lock_down(&mut score, &[Triple, Lock, Triple, Lock, Triple, Lock]);
        lock_down(&mut score, &[Single]);
        assert_eq!(score.level(), 2);
        assert_eq!(score.score, 500 * 3 + 100);
    }

    #[test]
    fn starting_on_a_higher_level_requires_the_lines_of_previous_levels() {
        // Starting on level 4 takes 40 lines to reach level 5
        let mut score = score_at_level(4);
        lock_down(&mut score, &[ScoreAction::Tetris; 9]);
        assert_eq!(score.level(), 4);
        lock_down(&mut score, &[ScoreAction::Tetris]);
        assert_eq!(score.level(), 5);
    }

    #[test]
    fn level_stops_at_15() {
        let mut score = score_at_level(15);
        lock_down(&mut score, &[ScoreAction::Tetris; 100]);
        assert_eq!(score.level(), 15);
    }

    #[test]
    fn t_spin_line_clears_count_towards_the_goal() {
        let mut score = score_at_level(1);
        use ScoreAction::*;
        lock_down(&mut score, &[TSpinTriple, TSpinDouble, TSpinSingle]);
        lock_down(&mut score, &[MiniTSpinSingle, MiniTSpin, TSpin]);
        assert_eq!(score.lines_cleared(), 7);
    }

    #[test]
    fn back_to_back_example_from_the_guideline() {
        // The example is at level 1, but its 11 lines would level up before the last action, so
        // play it at level 2 instead
        let mut score = score_at_level(2);
        use ScoreAction::*;
        // Plain lock downs in between so that no combo bonus is awarded
        lock_down(
            &mut score,
            &[Tetris, Lock, TSpinDouble, TSpin, Tetris, Lock, TSpinSingle],
        );
        assert_eq!(score.level(), 2);
        assert_eq!(
            score.score,
            (800 + (1200 + 600) + 400 + (800 + 400) + (800 + 400)) * 2
        );
    }

    #[test]
    fn back_to_back_bonus_is_level_dependent() {
        let mut score = score_at_level(3);
        lock_down(
            &mut score,
            &[ScoreAction::Tetris, ScoreAction::Lock, ScoreAction::Tetris],
        );
        assert_eq!(score.score, 800 * 3 + (800 * 3 + 400 * 3));
    }

    #[test]
    fn single_double_or_triple_breaks_back_to_back() {
        for breaker in [
            ScoreAction::Single,
            ScoreAction::Double,
            ScoreAction::Triple,
        ] {
            let mut score = score_at_level(1);
            lock_down(
                &mut score,
                &[ScoreAction::Tetris, ScoreAction::Lock, breaker],
            );
            lock_down(&mut score, &[ScoreAction::Lock, ScoreAction::Tetris]);
            assert_eq!(score.score, 800 + breaker.points() + 800, "{breaker:?}");
        }
    }

    #[test]
    fn t_spin_without_line_clear_does_not_start_back_to_back() {
        let mut score = score_at_level(1);
        lock_down(&mut score, &[ScoreAction::TSpin, ScoreAction::Tetris]);
        assert_eq!(score.score, 400 + 800);
    }

    #[test]
    fn mini_t_spin_single_continues_back_to_back() {
        let mut score = score_at_level(1);
        use ScoreAction::*;
        lock_down(&mut score, &[Tetris, Lock, MiniTSpinSingle, MiniTSpin]);
        assert_eq!(score.score, 800 + (200 + 100) + 100);
    }

    #[test]
    fn consecutive_line_clears_are_a_combo() {
        let mut score = score_at_level(2);
        use ScoreAction::*;
        lock_down(&mut score, &[Single, Double, Single, Triple]);
        let actions = (100 + 300 + 100 + 500) * 2;
        let combos = (50 + 100 + 150) * 2;
        assert_eq!(score.score, actions + combos);
    }

    #[test]
    fn lock_down_without_line_clear_ends_combo() {
        let mut score = score_at_level(1);
        use ScoreAction::*;
        lock_down(
            &mut score,
            &[Single, Single, MiniTSpin, Single, TSpin, Single],
        );
        assert_eq!(score.score, (100 + 150) + 100 + 100 + 400 + 100);
    }

    #[test]
    fn level_start_resets_back_to_back_and_combo() {
        let mut score = score_at_level(1);
        lock_down(&mut score, &[ScoreAction::Tetris]);
        score.handle_event(&ScoreEvent::LevelStart(1));
        lock_down(&mut score, &[ScoreAction::Tetris]);
        assert_eq!(score.score, 800);
    }

    #[test]
    fn mini_t_spin_double_counts_as_t_spin_double() {
        assert_eq!(
            ScoreAction::new(2, Some(TSpin::Mini)),
            Some(ScoreAction::TSpinDouble)
        );
        assert_eq!(ScoreAction::new(4, Some(TSpin::Full)), None);
    }
}
