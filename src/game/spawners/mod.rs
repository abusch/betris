use bevy::prelude::*;

use crate::{game::SKYLINE, model::Pos};

pub mod matrix;
pub mod next_zone;
pub mod piece;

pub use matrix::SpawnMatrix;
pub use next_zone::SpawnNextZone;
pub use piece::SpawnPiece;

/// Where the rotation point of a new piece is generated: the 5th column, in the first row above
/// the Skyline. The guideline counts cells from 1, i.e. (5, 21), but we count them from 0.
pub const INITIAL_POS: Pos = Pos::new(4, SKYLINE);

pub fn plugin(_app: &mut App) {
    // nothing yet
}

#[derive(Copy, Default, Clone, Component, Deref, DerefMut)]
pub struct Positioned(pub(crate) Pos);
