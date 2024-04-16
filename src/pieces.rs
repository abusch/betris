use bevy::prelude::*;
use num_enum::TryFromPrimitive;
use rand::thread_rng;

use crate::Pos;

const OFFSETS: [[[(isize, isize); 4]; 4]; 7] = [
    // O-Tetrimino
    [
        // North
        [(0, 0), (1, 0), (1, 1), (0, 1)],
        // East
        [(0, 0), (1, 0), (1, 1), (0, 1)],
        // South
        [(0, 0), (1, 0), (1, 1), (0, 1)],
        // West
        [(0, 0), (1, 0), (1, 1), (0, 1)],
    ],
    // I-Tetrimino
    [
        // North
        [(-1, 0), (0, 0), (1, 0), (2, 0)],
        // East
        [(1, -1), (1, 0), (1, 1), (1, 2)],
        // South
        [(-1, -1), (0, -1), (1, -1), (2, -1)],
        // West
        [(0, -1), (0, 0), (0, 1), (0, 2)],
    ],
    // T-Tetrimino
    [
        // North
        [(-1, 0), (0, 0), (1, 0), (0, 1)],
        // East
        [(0, 1), (0, 0), (0, -1), (1, 0)],
        // South
        [(1, 0), (0, 0), (-1, 0), (0, -1)],
        // West
        [(0, -1), (0, 0), (0, 1), (-1, 0)],
    ],
    // L-Tetrimino
    [
        // North
        [(-1, 0), (0, 0), (1, 0), (1, 1)],
        // East
        [(0, 1), (0, 0), (0, -1), (1, -1)],
        // South
        [(1, 0), (0, 0), (-1, 0), (-1, -1)],
        // West
        [(0, -1), (0, 0), (0, 1), (-1, 1)],
    ],
    // J-Tetrimino
    [
        // North
        [(-1, 1), (-1, 0), (0, 0), (1, 0)],
        // East
        [(1, 1), (0, 1), (0, 0), (0, -1)],
        // South
        [(1, -1), (1, 0), (0, 0), (-1, 0)],
        // West
        [(-1, -1), (0, -1), (0, 0), (0, 1)],
    ],
    // S-Tetrimino
    [
        // North
        [(-1, 0), (0, 0), (0, 1), (1, 1)],
        // East
        [(0, 1), (0, 0), (1, 0), (1, -1)],
        // South
        [(1, 0), (0, 0), (0, -1), (-1, -1)],
        // West
        [(0, -1), (0, 0), (-1, 0), (-1, 1)],
    ],
    // Z-Tetrimino
    [
        // North
        [(-1, 1), (0, 1), (0, 0), (1, 0)],
        // East
        [(1, 1), (1, 0), (0, 0), (0, -1)],
        // South
        [(1, -1), (0, -1), (0, 0), (-1, 0)],
        // West
        [(-1, -1), (-1, 0), (0, 0), (0, 1)],
    ],
];
const YELLOW: Color = Color::YELLOW;
const LIGHT_BLUE: Color = Color::BLUE;
const PURPLE: Color = Color::PURPLE;
const ORANGE: Color = Color::ORANGE;
const DARK_BLUE: Color = Color::MIDNIGHT_BLUE;
const GREEN: Color = Color::GREEN;
const RED: Color = Color::RED;

#[derive(Component, Debug, Clone, Copy)]
pub struct Piece {
    pub typ: Tetrimino,
    orientation: Facing,
}

impl Piece {
    pub fn new(typ: Tetrimino) -> Self {
        Self {
            typ,
            orientation: Facing::North,
        }
    }

    pub fn block_positions(&self, pos: Pos) -> [Pos; 4] {
        self.block_offsets()
            .map(|off| Pos::new(pos.x + off.0, pos.y + off.1))
    }

    pub fn block_offsets(&self) -> &[(isize, isize); 4] {
        &OFFSETS[self.typ as usize][self.orientation as usize]
    }

    pub fn rotate_cw(&mut self) {
        self.orientation = self.orientation.rotate_cw();
    }

    pub fn rotate_ccw(&mut self) {
        self.orientation = self.orientation.rotate_ccw();
    }

    pub fn color(&self) -> Color {
        match self.typ {
            Tetrimino::O => YELLOW,
            Tetrimino::I => LIGHT_BLUE,
            Tetrimino::T => PURPLE,
            Tetrimino::L => ORANGE,
            Tetrimino::J => DARK_BLUE,
            Tetrimino::S => GREEN,
            Tetrimino::Z => RED,
        }
    }

    pub fn min_y(&self, pos: Pos) -> isize {
        self.block_positions(pos)
            .iter()
            .map(|p| p.y)
            .min()
            .unwrap_or_default()
    }

    pub fn min_x(&self, pos: Pos) -> isize {
        self.block_positions(pos)
            .iter()
            .map(|p| p.x)
            .min()
            .unwrap_or_default()
    }

    pub fn max_x(&self, pos: Pos) -> isize {
        self.block_positions(pos)
            .iter()
            .map(|p| p.x)
            .max()
            .unwrap_or_default()
    }
}

impl From<Tetrimino> for Piece {
    fn from(value: Tetrimino) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Component, TryFromPrimitive)]
#[repr(usize)]
pub enum Tetrimino {
    O = 0,
    I,
    T,
    L,
    J,
    S,
    Z,
}

impl Tetrimino {
    #[allow(dead_code)]
    pub fn prev(&self) -> Self {
        Self::try_from_primitive((*self as usize + 6) % 7).unwrap()
    }

    #[allow(dead_code)]
    pub fn next(&self) -> Self {
        Self::try_from_primitive((*self as usize + 1) % 7).unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
#[repr(usize)]
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

#[derive(Resource)]
pub struct Bag(Vec<Tetrimino>);

impl Bag {
    pub fn new() -> Self {
        let mut bag = Self(Vec::with_capacity(7));
        bag.refill();
        bag
    }

    pub fn refill(&mut self) {
        use rand::seq::SliceRandom;
        self.0.clear();
        self.0.push(Tetrimino::O);
        self.0.push(Tetrimino::I);
        self.0.push(Tetrimino::T);
        self.0.push(Tetrimino::L);
        self.0.push(Tetrimino::J);
        self.0.push(Tetrimino::S);
        self.0.push(Tetrimino::Z);
        self.0.shuffle(&mut thread_rng());
    }

    pub fn pop_next(&mut self) -> Tetrimino {
        let next = self
            .0
            .pop()
            .expect("There should be at least one Tetrimino left in the bag!");
        if self.0.last().is_none() {
            self.refill();
        }

        next
    }

    pub fn peek_next(&self) -> Tetrimino {
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
