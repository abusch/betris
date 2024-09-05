use bevy::{
    ecs::{system::RunSystemOnce, world::Command},
    prelude::*,
    sprite::Anchor,
};

use crate::model::{Pos, Tetrimino};

use super::{Positioned, INITIAL_POS};

#[derive(Debug)]
pub struct SpawnPiece(pub Entity, pub Tetrimino, pub Pos, pub bool);

impl SpawnPiece {
    pub fn current(tetrimino: Tetrimino) -> Self {
        Self(Entity::PLACEHOLDER, tetrimino, INITIAL_POS, true)
    }

    pub fn next(tetrimino: Tetrimino) -> Self {
        Self(Entity::PLACEHOLDER, tetrimino, Pos::ZERO, false)
    }

    pub fn with_parent(self, parent: Entity) -> Self {
        Self(parent, self.1, self.2, self.3)
    }
}

impl Command for SpawnPiece {
    fn apply(self, world: &mut World) {
        world.run_system_once_with(self, spawn);
    }
}

fn spawn(In(config): In<SpawnPiece>, mut commands: Commands) {
    info!("Spawning piece");
    let SpawnPiece(parent, piece, pos, is_current) = config;

    commands.entity(parent).with_children(|children| {
        let mut builder = children.spawn(PieceBundle {
            spatial: SpatialBundle {
                transform: Transform::from_xyz(pos.x as f32, pos.y as f32, 1.0),
                ..default()
            },
            piece,
            pos: Positioned(pos),
        });
        if is_current {
            builder.insert((Name::new("Current piece"), CurrentPiece));
        } else {
            builder.insert(Name::new("Next piece"));
        }
        builder.with_children(|children| {
            for p in piece.block_positions(&Pos::ZERO) {
                children.spawn(MinoBundle::new(p, piece.kind.color()));
            }
        });
    });
}

#[derive(Bundle)]
pub struct PieceBundle {
    spatial: SpatialBundle,
    piece: Tetrimino,
    pos: Positioned,
}

/// Marker component for the current piece (i.e. the piece controlled by the player)
#[derive(Component)]
pub struct CurrentPiece;

/// A mino (i.e block) which is part of a piece
#[derive(Component)]
pub struct Mino;

#[derive(Bundle)]
pub struct MinoBundle {
    sprite: SpriteBundle,
    mino: Mino,
}

impl MinoBundle {
    pub fn new(pos: Pos, color: Color) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(1.0)),
                    anchor: Anchor::Center,
                    color,
                    ..default()
                },
                transform: Transform::from_xyz(pos.x as f32, pos.y as f32, 1.0),
                ..default()
            },
            mino: Mino,
        }
    }
}
