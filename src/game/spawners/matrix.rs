use bevy::{prelude::*, sprite::Anchor};
use bevy_enhanced_input::prelude::Actions;

use crate::{
    game::{MATRIX_WIDTH, SCALE, input::InGame},
    screen::Screen,
};

/// Main game area. Despawning it also despawns the blocks and pieces in it.
pub fn matrix() -> impl Scene {
    let transform = Transform::from_xyz(-10.0 * SCALE, -11.0 * SCALE, 1.0)
        .with_scale(Vec3::new(SCALE, SCALE, 1.0));
    let width = MATRIX_WIDTH as f32;
    bsn! {
        Name("Matrix")
        DespawnOnExit<Screen>(Screen::Gameplay)
        transform
        Visibility
        Actions<InGame>
        Children [
            // Floor is 1 unit below "zero"
            @wall(width, 1.0, 0.0, -1.0)
            Name("Bottom wall")
            --
            @wall(width, 1.0, 0.0, 22.0)
            Name("Top wall")
            --
            @wall(1.0, 24.0, -1.0, -1.0)
            Name("Left wall")
            --
            @wall(1.0, 24.0, width, -1.0)
            Name("Right wall")
        ]
    }
}

/// A wall of the given size, with its bottom left corner at the given position
fn wall(width: f32, height: f32, x: f32, y: f32) -> impl Scene {
    let anchor = Anchor::BOTTOM_LEFT;
    let transform = Transform::from_xyz(x, y, 1.0);
    bsn! {
        Sprite { custom_size: {Some(Vec2::new(width, height))} }
        anchor
        transform
    }
}
