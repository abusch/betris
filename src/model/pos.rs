use std::{fmt::Display, ops::Add};

use crate::game::MATRIX_WIDTH;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos {
    pub x: i8,
    pub y: i8,
}

impl Pos {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }

    /// Return the position to the left of the current one
    #[must_use]
    pub fn left(&self) -> Self {
        Self {
            x: self.x - 1,
            ..*self
        }
    }

    /// Return the position to the right of the current one
    #[must_use]
    pub fn right(&self) -> Self {
        Self {
            x: self.x + 1,
            ..*self
        }
    }

    /// Return the position below the current one
    #[must_use]
    pub fn down(&self) -> Self {
        Self {
            y: self.y - 1,
            ..*self
        }
    }

    pub fn to_index(self) -> usize {
        ((self.y as u8) * MATRIX_WIDTH + self.x as u8) as usize
    }

    pub fn from_index(index: usize) -> Self {
        let x = (index % MATRIX_WIDTH as usize) as i8;
        let y = (index / MATRIX_WIDTH as usize) as i8;
        Self::new(x, y)
    }
}

impl From<(i8, i8)> for Pos {
    fn from(value: (i8, i8)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Add<Pos> for Pos {
    type Output = Pos;

    fn add(self, rhs: Pos) -> Self::Output {
        Pos::new(self.x + rhs.x, self.y + rhs.y)
    }
}
