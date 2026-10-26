use bevy::prelude::*;

use crate::{game::SCALE, screen::Screen};

/// Hold Queue display zone, to the left of the matrix. Despawning it also despawns the held piece.
pub fn hold_zone() -> impl Scene {
    let transform =
        Transform::from_xyz(-340.0, 100.0, 1.0).with_scale(Vec3::new(SCALE, SCALE, 1.0));
    bsn! {
        Name("Hold queue zone")
        DespawnOnExit<Screen>(Screen::Gameplay)
        transform
        Visibility
        HoldQueueZone
    }
}

/// The parent component of where the held piece is displayed
#[derive(Default, Clone, Component)]
pub struct HoldQueueZone;
