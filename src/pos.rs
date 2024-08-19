use std::fmt::Display;

use bevy::prelude::*;

use crate::game::MATRIX_WIDTH;

#[derive(Component, Clone, Copy, Debug)]
pub struct Pos {
    pub x: isize,
    pub y: isize,
}

impl Pos {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(x: isize, y: isize) -> Self {
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
        (self.y as usize) * MATRIX_WIDTH + self.x as usize
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
