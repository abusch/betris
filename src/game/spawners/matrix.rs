use bevy::{prelude::*, sprite::Anchor};
use bevy_enhanced_input::prelude::Actions;

use crate::{
    game::{MATRIX_WIDTH, SCALE, input::InGame},
    screen::Screen,
};

/// Height of the visible part of the Matrix, in blocks
const VISIBLE_HEIGHT: f32 = 22.0;
/// Bright enough to bloom
const FRAME_COLOR: Color = Color::LinearRgba(LinearRgba::rgb(0.3, 1.2, 3.0));
/// Thickness of the frame's lines, in blocks (2 pixels)
const FRAME_THICKNESS: f32 = 2.0 / SCALE;
/// How far outside the Matrix the frame's lines are, in blocks
const FRAME_OFFSETS: [f32; 2] = [4.0 / SCALE, 11.0 / SCALE];
const PANEL_COLOR: Color = Color::Srgba(Srgba::rgb(0.03, 0.03, 0.08));
const DOT_COLOR: Color = Color::LinearRgba(LinearRgba::rgb(0.04, 0.1, 0.3));
/// Size of the dot marking each empty cell, in blocks
const DOT_SIZE: f32 = 3.0 / SCALE;

/// Main game area. Despawning it also despawns the blocks and pieces in it.
pub fn matrix() -> impl Scene {
    let transform = Transform::from_xyz(-5.0 * SCALE, -11.0 * SCALE, 1.0)
        .with_scale(Vec3::new(SCALE, SCALE, 1.0));
    let width = MATRIX_WIDTH as f32;
    let height = VISIBLE_HEIGHT;

    // A dark panel behind the Matrix, with a dot in the middle of each cell
    let mut decorations = vec![rect(
        Vec2::ZERO,
        Vec2::new(width, height),
        PANEL_COLOR,
        -0.2,
    )];
    for x in 0..MATRIX_WIDTH {
        for y in 0..height as u8 {
            let center = Vec2::new(f32::from(x), f32::from(y)) + 0.5;
            decorations.push(rect(
                center - DOT_SIZE / 2.0,
                Vec2::splat(DOT_SIZE),
                DOT_COLOR,
                -0.1,
            ));
        }
    }
    // A double neon line around the Matrix
    for offset in FRAME_OFFSETS {
        let (min, max) = (Vec2::splat(-offset), Vec2::new(width, height) + offset);
        let (t, size) = (FRAME_THICKNESS, max - min);
        decorations.extend([
            rect(min - t, Vec2::new(size.x + 2.0 * t, t), FRAME_COLOR, 0.0),
            rect(
                Vec2::new(min.x - t, max.y),
                Vec2::new(size.x + 2.0 * t, t),
                FRAME_COLOR,
                0.0,
            ),
            rect(min - t, Vec2::new(t, size.y + 2.0 * t), FRAME_COLOR, 0.0),
            rect(
                Vec2::new(max.x, min.y - t),
                Vec2::new(t, size.y + 2.0 * t),
                FRAME_COLOR,
                0.0,
            ),
        ]);
    }

    bsn! {
        Name("Matrix")
        DespawnOnExit<Screen>(Screen::Gameplay)
        transform
        Visibility
        Actions<InGame>
        Children [ {decorations} ]
    }
}

/// A rectangle of the given size and color, with its bottom left corner at the given position
fn rect(pos: Vec2, size: Vec2, color: Color, z: f32) -> impl Scene {
    let anchor = Anchor::BOTTOM_LEFT;
    let transform = Transform::from_translation(pos.extend(z));
    bsn! {
        Sprite {
            custom_size: {Some(size)},
            color: color,
        }
        anchor
        transform
    }
}
