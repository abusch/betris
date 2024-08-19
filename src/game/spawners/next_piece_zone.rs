use bevy::prelude::*;

use super::SpawnNextPieceZone;
use crate::game::SCALE;

pub fn spawn(_trigger: Trigger<SpawnNextPieceZone>, mut commands: Commands) {
    // Next-piece display zone
    commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(100.0, 100.0, 1.0).with_scale(Vec3::new(
                SCALE / 2.0,
                SCALE / 2.0,
                1.0,
            )),
            ..default()
        },
        NextPieceDisplay,
    ));
}

/// The parent component of where the next piece is displayed
#[derive(Component)]
pub struct NextPieceDisplay;
