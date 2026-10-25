use bevy::prelude::*;

use crate::{game::SCALE, screen::Screen};

/// Next-piece display zone. Despawning it also despawns the next piece.
pub fn next_zone() -> impl Scene {
    let transform = Transform::from_xyz(100.0, 100.0, 1.0).with_scale(Vec3::new(SCALE, SCALE, 1.0));
    bsn! {
        Name("Next tetrimino zone")
        DespawnOnExit<Screen>(Screen::Gameplay)
        transform
        Visibility
        NextTetriminoZone
    }
}

/// The parent component of where the next piece is displayed
#[derive(Default, Clone, Component)]
pub struct NextTetriminoZone;
