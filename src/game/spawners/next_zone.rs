use bevy::prelude::*;

use super::SpawnNextZone;
use crate::game::SCALE;

pub fn spawn(_trigger: Trigger<SpawnNextZone>, mut commands: Commands) {
    // Next-piece display zone
    commands.spawn((
        Name::new("Next tetrimino zone"),
        SpatialBundle {
            transform: Transform::from_xyz(100.0, 100.0, 1.0)
                .with_scale(Vec3::new(SCALE, SCALE, 1.0)),
            ..default()
        },
        NextTetriminoZone,
    ));
}

/// The parent component of where the next piece is displayed
#[derive(Component)]
pub struct NextTetriminoZone;
