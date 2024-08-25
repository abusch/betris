use bevy::prelude::*;

use crate::{screen::Screen, AppSet};

pub fn plugin(app: &mut App) {
    app.init_resource::<Score>()
        .register_type::<Score>()
        .add_event::<ScoreEvent>()
        .add_systems(OnEnter(Screen::Playing), setup)
        .add_systems(
            Update,
            update
                .run_if(in_state(Screen::Playing))
                .in_set(AppSet::Update),
        )
        .add_systems(OnExit(Screen::Playing), cleanup);
}

#[derive(Default, Resource, Reflect)]
pub struct Score {
    level: u64,
    score: u64,
}

impl Score {
    /// Add the given number of points applying the level factor
    pub fn add_with_mult(&mut self, points: u64) {
        self.score += points * self.level;
    }

    pub fn handle_event(&mut self, event: &ScoreEvent) {
        match event {
            ScoreEvent::LevelStart(level) => {
                self.level = *level as u64;
                self.score = 0;
            }
            ScoreEvent::Single => self.add_with_mult(100),
            ScoreEvent::Double => self.add_with_mult(300),
            ScoreEvent::Triple => self.add_with_mult(500),
            ScoreEvent::Tetris => self.add_with_mult(800),
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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Event)]
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

fn setup() {}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Score>();
}

fn update(mut score: ResMut<Score>, mut events: EventReader<ScoreEvent>) {
    for event in events.read() {
        score.handle_event(event)
    }
}
