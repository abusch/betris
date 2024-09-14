use bevy::prelude::*;

use crate::model::Pos;

pub mod matrix;
pub mod next_zone;
pub mod piece;

pub use matrix::SpawnMatrix;
pub use next_zone::SpawnNextZone;
pub use piece::SpawnPiece;

pub const INITIAL_POS: Pos = Pos::new(5, 21);

pub fn plugin(_app: &mut App) {
    // nothing yet
}

#[derive(Copy, Clone, Component, Deref, DerefMut)]
pub struct Positioned(pub(crate) Pos);
