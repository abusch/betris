use bevy::prelude::*;

use crate::{pieces::Piece, pos::Pos};

pub mod matrix;
pub mod next_piece_zone;
pub mod piece;

pub const INITIAL_POS: Pos = Pos::new(5, 21);

pub fn plugin(app: &mut App) {
    app.observe(matrix::spawn)
        .observe(next_piece_zone::spawn)
        .observe(piece::spawn);
}

#[derive(Event)]
pub struct SpawnMatrix;

#[derive(Event)]
pub struct SpawnPiece(pub Piece, pub Pos, pub bool);

impl SpawnPiece {
    pub fn current(piece: Piece) -> Self {
        Self(piece, INITIAL_POS, true)
    }

    pub fn next(piece: Piece) -> Self {
        Self(piece, Pos::ZERO, false)
    }
}

#[derive(Event)]
pub struct SpawnNextPieceZone;
