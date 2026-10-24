use bevy::{ecs::system::RunSystemOnce, prelude::*};

use crate::{game::SCALE, screen::Screen};

#[derive(Debug)]
pub struct SpawnNextZone;

impl Command for SpawnNextZone {
    type Out = ();

    fn apply(self, world: &mut World) {
        world
            .run_system_once_with(spawn, self)
            .expect("Failed to spawn NextZone");
    }
}

fn spawn(In(_): In<SpawnNextZone>, mut commands: Commands) {
    // Next-piece display zone. Despawning it also despawns the next piece.
    commands.spawn((
        Name::new("Next tetrimino zone"),
        DespawnOnExit(Screen::Gameplay),
        Transform::from_xyz(100.0, 100.0, 1.0).with_scale(Vec3::new(SCALE, SCALE, 1.0)),
        Visibility::default(),
        NextTetriminoZone,
    ));
}

/// The parent component of where the next piece is displayed
#[derive(Component)]
pub struct NextTetriminoZone;
