use bevy::{ecs::system::RunSystemOnce, prelude::*, sprite::Anchor};
use bevy_enhanced_input::prelude::Actions;

use crate::game::{GameState, MATRIX_WIDTH, SCALE, input::InGame};

#[derive(Debug)]
pub struct SpawnMatrix;

impl Command for SpawnMatrix {
    fn apply(self, world: &mut World) {
        world
            .run_system_once_with(spawn, self)
            .expect("Failed to spawn Matrix");
    }
}

fn spawn(In(_): In<SpawnMatrix>, mut commands: Commands, mut state: ResMut<GameState>) {
    // Matrix i.e main game area
    state.matrix.root_entity = commands
        .spawn((
            Name::new("Matrix"),
            Transform::from_xyz(-10.0 * SCALE, -11.0 * SCALE, 1.0)
                .with_scale(Vec3::new(SCALE, SCALE, 1.0)),
            Visibility::default(),
            Actions::<InGame>::default(),
            children![
                (
                    Name::new("Bottom wall"),
                    Sprite {
                        custom_size: Some(Vec2::new(MATRIX_WIDTH as f32, 1.0)),
                        anchor: Anchor::BottomLeft,
                        ..default()
                    },
                    // Floor is 1 unit below "zero"
                    Transform::from_xyz(0.0, -1.0, 1.0),
                ),
                (
                    Name::new("Top wall"),
                    Sprite {
                        custom_size: Some(Vec2::new(MATRIX_WIDTH as f32, 1.0)),
                        anchor: Anchor::BottomLeft,
                        ..default()
                    },
                    Transform::from_xyz(0.0, 22.0, 1.0),
                ),
                (
                    Name::new("Left wall"),
                    Sprite {
                        custom_size: Some(Vec2::new(1.0, 24.0)),
                        anchor: Anchor::BottomLeft,
                        ..default()
                    },
                    Transform::from_xyz(-1.0, -1.0, 1.0),
                ),
                (
                    Name::new("Right wall"),
                    Sprite {
                        custom_size: Some(Vec2::new(1.0, 24.0)),
                        anchor: Anchor::BottomLeft,
                        ..default()
                    },
                    Transform::from_xyz(MATRIX_WIDTH as f32, -1.0, 1.0),
                )
            ],
        ))
        .id();
}
