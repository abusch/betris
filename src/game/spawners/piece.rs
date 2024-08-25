use bevy::{prelude::*, sprite::Anchor};

use crate::{pieces::Piece, pos::Pos};

use super::{Positioned, SpawnPiece};

pub fn spawn(trigger: Trigger<SpawnPiece>, mut commands: Commands) {
    info!("Spawning piece");
    let parent = trigger.entity();
    let SpawnPiece(piece, pos, is_current) = trigger.event();

    commands.entity(parent).with_children(|children| {
        let mut builder = children.spawn(PieceBundle {
            spatial: SpatialBundle {
                transform: Transform::from_xyz(pos.x as f32, pos.y as f32, 1.0),
                ..default()
            },
            piece: *piece,
            pos: Positioned(*pos),
        });
        if *is_current {
            builder.insert(CurrentPiece);
        }
        builder.with_children(|children| {
            for p in piece.block_positions(&Pos::ZERO) {
                children.spawn(MinoBundle::new(p, piece.color()));
            }
        });
    });
}

#[derive(Bundle)]
pub struct PieceBundle {
    spatial: SpatialBundle,
    piece: Piece,
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
