use bevy::{
    ecs::{system::RunSystemOnce, world::Command},
    prelude::*,
    sprite::Anchor,
};

use crate::game::{GameState, SCALE};

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
                transform: Transform::from_xyz(-200.0, -200.0, 1.0)
                    .with_scale(Vec3::new(SCALE, SCALE, 1.0)),
                ..default()
            },
        ))
        .with_children(|children| {
            // "floor"
            children.spawn(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(10.0, 1.0)),
                    anchor: Anchor::CenterLeft,
                    ..default()
                },
                // Floor is 1 unit below "zero"
                transform: Transform::from_xyz(0.0, -1.0, 1.0),
                ..default()
            });
        })
        .id();
}
