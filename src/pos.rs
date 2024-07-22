use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug)]
pub struct Pos {
    pub x: isize,
    pub y: isize,
}

impl Pos {
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
}
