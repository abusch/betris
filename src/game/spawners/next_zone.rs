use bevy::{
    ecs::{system::RunSystemOnce, world::Command},
    prelude::*,
};

use crate::game::SCALE;

#[derive(Debug)]
pub struct SpawnNextZone;

impl Command for SpawnNextZone {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(self, spawn);
    }
}

fn spawn(In(_): In<SpawnNextZone>, mut commands: Commands) {
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
