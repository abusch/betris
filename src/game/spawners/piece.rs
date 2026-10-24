use bevy::{prelude::*, sprite::Anchor};

use crate::model::{Pos, Tetrimino};

use super::Positioned;

/// Opacity of the ghost piece's minos
const GHOST_ALPHA: f32 = 0.3;

/// The piece controlled by the player
pub fn current_piece(tetrimino: Tetrimino, pos: Pos) -> impl Scene {
    bsn! {
        @piece(tetrimino, pos, tetrimino.kind.color())
        Name("Current piece")
        CurrentPiece
    }
}

/// Shows where the current piece would land
pub fn ghost_piece(tetrimino: Tetrimino, pos: Pos) -> impl Scene {
    bsn! {
        @piece(tetrimino, pos, tetrimino.kind.color().with_alpha(GHOST_ALPHA))
        Name("Ghost piece")
        GhostPiece
        // Keep the ghost behind the current piece so that the current piece is always drawn on top
        // when they overlap
        Transform { translation: {Vec3::new(pos.x as f32, pos.y as f32, 0.5)} }
    }
}

/// The piece shown in the next-piece zone
pub fn next_piece(tetrimino: Tetrimino) -> impl Scene {
    bsn! {
        @piece(tetrimino, Pos::ZERO, tetrimino.kind.color())
        Name("Next piece")
    }
}

fn piece(tetrimino: Tetrimino, pos: Pos, color: Color) -> impl Scene {
    let minos: Vec<_> = tetrimino
        .block_offsets()
        .iter()
        .map(|&offset| mino(offset, color))
        .collect();
    bsn! {
        Piece
        tetrimino
        Positioned(pos)
        Transform { translation: {Vec3::new(pos.x as f32, pos.y as f32, 1.0)} }
        Children [ {minos} ]
    }
}

/// A mino of the given color, at the given position relative to its parent
pub fn mino(pos: Pos, color: Color) -> impl Scene {
    let anchor = Anchor::BOTTOM_LEFT;
    let transform = Transform::from(pos);
    bsn! {
        Mino
        Sprite { color: color, custom_size: {Some(Vec2::ONE)} }
        anchor
        transform
    }
}

#[derive(Default, Clone, Component)]
#[require(Tetrimino, Positioned, Transform, Visibility)]
pub struct Piece;

/// Marker component for the current piece (i.e. the piece controlled by the player)
#[derive(Default, Clone, Component)]
pub struct CurrentPiece;

/// Marker component for the ghost piece
#[derive(Default, Clone, Component)]
pub struct GhostPiece;

/// A mino (i.e block) which is part of a piece
#[derive(Default, Clone, Component)]
pub struct Mino;
