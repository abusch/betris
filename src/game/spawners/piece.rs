use bevy::{prelude::*, sprite::Anchor};

use super::Positioned;
use crate::{
    game::intensities::Intensities,
    model::{Pos, Tetrimino},
};

/// Neon outline texture, tinted by the mino's color
const MINO_IMAGE: &str = "images/mino.png";
/// Just the outline of `MINO_IMAGE`, for the ghost piece
const GHOST_MINO_IMAGE: &str = "images/mino_outline.png";
/// Opacity of the held piece's minos while Hold is unavailable
pub const HELD_UNAVAILABLE_ALPHA: f32 = 0.3;

/// The piece controlled by the player
pub fn current_piece(tetrimino: Tetrimino, pos: Pos, intensities: Intensities) -> impl Scene {
    let color = intensities.piece_color(tetrimino.kind);
    bsn! {
        @piece(tetrimino, pos, color, MINO_IMAGE)
        Name("Current piece")
        CurrentPiece
    }
}

/// Shows where the current piece would land
pub fn ghost_piece(tetrimino: Tetrimino, pos: Pos, intensities: Intensities) -> impl Scene {
    let color = intensities.ghost_color();
    bsn! {
        @piece(tetrimino, pos, color, GHOST_MINO_IMAGE)
        Name("Ghost piece")
        GhostPiece
        // Keep the ghost behind the current piece so that the current piece is always drawn on top
        // when they overlap
        Transform { translation: {Vec3::new(pos.x as f32, pos.y as f32, 0.5)} }
    }
}

/// The piece shown in the next-piece zone
pub fn next_piece(tetrimino: Tetrimino, intensities: Intensities) -> impl Scene {
    let color = intensities.piece_color(tetrimino.kind);
    bsn! {
        @piece(tetrimino, Pos::ZERO, color, MINO_IMAGE)
        Name("Next piece")
    }
}

/// The piece shown in the Hold Queue
pub fn held_piece(tetrimino: Tetrimino, intensities: Intensities) -> impl Scene {
    let color = intensities.piece_color(tetrimino.kind);
    bsn! {
        @piece(tetrimino, Pos::ZERO, color, MINO_IMAGE)
        Name("Held piece")
        HeldPiece
    }
}

/// A piece whose minos show `image` tinted with `color`
fn piece(tetrimino: Tetrimino, pos: Pos, color: Color, image: &'static str) -> impl Scene {
    let minos: Vec<_> = tetrimino
        .block_offsets()
        .iter()
        .map(|&offset| mino_with_image(offset, color, image))
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
    mino_with_image(pos, color, MINO_IMAGE)
}

/// A mino showing `image` tinted with `color`, at the given position relative to its parent
fn mino_with_image(pos: Pos, color: Color, image: &'static str) -> impl Scene {
    let anchor = Anchor::BOTTOM_LEFT;
    let transform = Transform::from(pos);
    bsn! {
        Mino
        Sprite { image: image, color: color, custom_size: {Some(Vec2::ONE)} }
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

/// Marker component for the piece in the Hold Queue
#[derive(Default, Clone, Component)]
pub struct HeldPiece;

/// A mino (i.e block) which is part of a piece
#[derive(Default, Clone, Component)]
pub struct Mino;
