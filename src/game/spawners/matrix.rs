use bevy::{
    ecs::{system::RunSystemOnce, world::Command},
    prelude::*,
    sprite::Anchor,
};

use crate::game::{GameState, MATRIX_WIDTH, SCALE};

#[derive(Debug)]
pub struct SpawnMatrix;

impl Command for SpawnMatrix {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(self, spawn);
    }
}

fn spawn(In(_): In<SpawnMatrix>, mut commands: Commands, mut state: ResMut<GameState>) {
    // Matrix i.e main game area
    state.matrix.root_entity = commands
        .spawn((
            Name::new("Matrix"),
            SpatialBundle {
                transform: Transform::from_xyz(-10.0 * SCALE, -11.0 * SCALE, 1.0)
                    .with_scale(Vec3::new(SCALE, SCALE, 1.0)),
                ..default()
            },
        ))
        .with_children(|children| {
            // "floor"
            children.spawn((
                Name::new("Bottom wall"),
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(MATRIX_WIDTH as f32, 1.0)),
                        anchor: Anchor::BottomLeft,
                        ..default()
                    },
                    // Floor is 1 unit below "zero"
                    transform: Transform::from_xyz(0.0, -1.0, 1.0),
                    ..default()
                },
            ));
            // "ceiling"
            children.spawn((
                Name::new("Top wall"),
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(MATRIX_WIDTH as f32, 1.0)),
                        anchor: Anchor::BottomLeft,
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 22.0, 1.0),
                    ..default()
                },
            ));
            // "left wall"
            children.spawn((
                Name::new("Left wall"),
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(1.0, 24.0)),
                        anchor: Anchor::BottomLeft,
                        ..default()
                    },
                    transform: Transform::from_xyz(-1.0, -1.0, 1.0),
                    ..default()
                },
            ));
            // "Right wall"
            children.spawn((
                Name::new("Right wall"),
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(1.0, 24.0)),
                        anchor: Anchor::BottomLeft,
                        ..default()
                    },
                    transform: Transform::from_xyz(MATRIX_WIDTH as f32, -1.0, 1.0),
                    ..default()
                },
            ));
        })
        .id();
}
