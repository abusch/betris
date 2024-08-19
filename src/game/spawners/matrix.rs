use bevy::{prelude::*, sprite::Anchor};

use crate::game::{GameState, SCALE};

use super::SpawnMatrix;

pub fn spawn(_trigger: Trigger<SpawnMatrix>, mut commands: Commands, mut state: ResMut<GameState>) {
    // Matrix i.e main game area
    state.matrix.root_entity = commands
        .spawn((SpatialBundle {
            transform: Transform::from_xyz(-200.0, -200.0, 1.0)
                .with_scale(Vec3::new(SCALE, SCALE, 1.0)),
            ..default()
        },))
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
