use bevy::{color::palettes, prelude::*};
use num_enum::TryFromPrimitive;
use rand::thread_rng;

use super::{data::OFFSETS, pos::Pos};

const YELLOW: Srgba = palettes::css::YELLOW;
const LIGHT_BLUE: Srgba = palettes::css::BLUE;
const PURPLE: Srgba = palettes::css::PURPLE;
const ORANGE: Srgba = palettes::css::ORANGE;
const DARK_BLUE: Srgba = palettes::css::MIDNIGHT_BLUE;
const GREEN: Srgba = palettes::css::GREEN;
const RED: Srgba = palettes::css::RED;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tetrimino {
    pub kind: TetriminoKind,
    pub facing: Facing,
}

impl Tetrimino {
    pub fn new(typ: TetriminoKind) -> Self {
        Self {
            kind: typ,
            facing: Facing::North,
        }
    }

    pub fn block_positions(&self, pos: &Pos) -> [Pos; 4] {
        self.block_offsets().map(|off| *pos + off)
    }

    pub fn block_offsets(&self) -> &[Pos; 4] {
        &OFFSETS[self.kind as usize][self.facing as usize]
    }

    pub fn rotate_cw(&mut self) {
        self.facing = self.facing.rotate_cw();
    }

    pub fn rotate_ccw(&mut self) {
        self.facing = self.facing.rotate_ccw();
    }

    pub fn rotated_cw(&self) -> Self {
        let mut rotated = *self;
        rotated.rotate_cw();
        rotated
    }

    pub fn rotated_ccw(&self) -> Self {
        let mut rotated = *self;
        rotated.rotate_ccw();
        rotated
    }

    pub fn min_y(&self, pos: &Pos) -> i8 {
        self.block_positions(pos)
            .iter()
            .map(|p| p.y)
            .min()
            .unwrap_or_default()
    }

    pub fn min_x(&self, pos: &Pos) -> i8 {
        self.block_positions(pos)
            .iter()
            .map(|p| p.x)
            .min()
            .unwrap_or_default()
    }

    pub fn max_x(&self, pos: &Pos) -> i8 {
        self.block_positions(pos)
            .iter()
            .map(|p| p.x)
            .max()
            .unwrap_or_default()
    }
}

impl From<TetriminoKind> for Tetrimino {
    fn from(value: TetriminoKind) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, TryFromPrimitive, Reflect)]
#[repr(u8)]
pub enum TetriminoKind {
    O = 0,
    I,
    T,
    L,
    J,
    S,
    Z,
}

impl TetriminoKind {
    pub fn color(&self) -> Color {
        match self {
            TetriminoKind::O => YELLOW.into(),
            TetriminoKind::I => LIGHT_BLUE.into(),
            TetriminoKind::T => PURPLE.into(),
            TetriminoKind::L => ORANGE.into(),
            TetriminoKind::J => DARK_BLUE.into(),
            TetriminoKind::S => GREEN.into(),
            TetriminoKind::Z => RED.into(),
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            TetriminoKind::O,
            TetriminoKind::I,
            TetriminoKind::T,
            TetriminoKind::L,
            TetriminoKind::J,
            TetriminoKind::S,
            TetriminoKind::Z,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
#[repr(u8)]
pub enum Facing {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Facing {
    pub fn rotate_cw(&self) -> Self {
        match self {
            Facing::North => Facing::East,
            Facing::East => Facing::South,
            Facing::South => Facing::West,
            Facing::West => Facing::North,
        }
    }

    pub fn rotate_ccw(&self) -> Self {
        match self {
            Facing::North => Facing::West,
            Facing::East => Facing::North,
            Facing::South => Facing::East,
            Facing::West => Facing::South,
        }
    }
}

#[derive(Reflect)]
pub struct Bag(Vec<TetriminoKind>);

impl Bag {
    pub fn new() -> Self {
        let mut bag = Self(Vec::with_capacity(7));
        bag.refill();
        bag
    }

    pub fn refill(&mut self) {
        use rand::seq::SliceRandom;
        self.0.clear();
        self.0.extend_from_slice(TetriminoKind::all());
        self.0.shuffle(&mut thread_rng());
    }

    pub fn pop_next(&mut self) -> TetriminoKind {
        let next = self
            .0
            .pop()
            .expect("There should be at least one Tetrimino left in the bag!");
        if self.0.last().is_none() {
            self.refill();
        }

        next
    }

    pub fn peek_next(&self) -> TetriminoKind {
        self.0
            .last()
            .copied()
            .expect("There should be at least one Tetrimino left in the bag!")
    }
}

impl Default for Bag {
    fn default() -> Self {
        Self::new()
    }
}
