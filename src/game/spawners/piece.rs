use bevy::prelude::*;

use crate::{game::MinoBundle, pieces::Piece, pos::Pos};

use super::SpawnPiece;

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
            pos: *pos,
        });
        if *is_current {
            builder.insert(CurrentPiece);
        }
        builder.with_children(|children| {
            for p in piece.block_positions(Pos::ZERO) {
                children.spawn(MinoBundle::new(p, piece.color()));
            }
        });
    });
}

#[derive(Bundle)]
pub struct PieceBundle {
    spatial: SpatialBundle,
    piece: Piece,
    pos: Pos,
}

/// Marker component for the current piece (i.e. the piece controlled by the player)
#[derive(Component)]
pub struct CurrentPiece;
