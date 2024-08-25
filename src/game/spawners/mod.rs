use bevy::prelude::*;

use crate::model::{Pos, Tetrimino};

pub mod matrix;
pub mod next_zone;
pub mod piece;

const INITIAL_POS: Pos = Pos::new(5, 21);

pub fn plugin(app: &mut App) {
    app.observe(matrix::spawn)
        .observe(next_zone::spawn)
        .observe(piece::spawn);
}

#[derive(Copy, Clone, Component, Deref, DerefMut)]
pub struct Positioned(pub(crate) Pos);

#[derive(Event)]
pub struct SpawnMatrix;

#[derive(Event)]
pub struct SpawnPiece(pub Tetrimino, pub Pos, pub bool);

impl SpawnPiece {
    pub fn current(tetrimino: Tetrimino) -> Self {
        Self(tetrimino, INITIAL_POS, true)
    }

    pub fn next(tetrimino: Tetrimino) -> Self {
        Self(tetrimino, Pos::ZERO, false)
    }
}

#[derive(Event)]
pub struct SpawnNextZone;
