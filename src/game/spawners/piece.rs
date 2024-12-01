use bevy::{
    color::palettes::basic::AQUA,
    ecs::{
        system::{EntityCommand, RunSystemOnce},
        world::Command,
    },
    prelude::*,
};
use bevy_vector_shapes::{
    prelude::{BuildShapeChildren, ShapeConfig},
    shapes::{RectangleSpawner, ThicknessType},
};

use crate::model::{Pos, Tetrimino};

use super::{Positioned, INITIAL_POS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    Current,
    Ghost,
    Next,
}

#[derive(Debug)]
pub struct SpawnPiece(pub Entity, pub Tetrimino, pub Pos, pub PieceType);

impl SpawnPiece {
    pub fn current(tetrimino: Tetrimino) -> Self {
        Self(
            Entity::PLACEHOLDER,
            tetrimino,
            INITIAL_POS,
            PieceType::Current,
        )
    }

    pub fn ghost(tetrimino: Tetrimino, pos: Pos) -> Self {
        Self(Entity::PLACEHOLDER, tetrimino, pos, PieceType::Ghost)
    }

    pub fn next(tetrimino: Tetrimino) -> Self {
        Self(Entity::PLACEHOLDER, tetrimino, Pos::ZERO, PieceType::Next)
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
    let SpawnPiece(parent, piece, pos, piece_type) = config;

    // Spawn a PieceBundle as a child of the given parent entity
    commands.entity(parent).with_children(|children| {
        let mut builder = children.spawn(PieceBundle {
            spatial: SpatialBundle {
                transform: Transform::from_xyz(pos.x as f32, pos.y as f32, 1.0),
                ..default()
            },
            piece,
            pos: Positioned(pos),
        });
        match piece_type {
            PieceType::Current => {
                builder.insert((Name::new("Current piece"), CurrentPiece));
            }
            PieceType::Ghost => {
                builder.insert((Name::new("Ghost piece"), GhostPiece));
            }
            PieceType::Next => {
                builder.insert(Name::new("Next piece"));
            }
        }

        let color = (piece_type != PieceType::Ghost).then_some(piece.kind.color());
        // Spawn Mino entities as children of the new PieceBundle entity
        builder.with_children(|c| {
            for p in piece.block_offsets() {
                c.spawn_empty().add(SpawnMino(*p, color));
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

/// Marker component for the ghost piece
#[derive(Component)]
pub struct GhostPiece;

/// A mino (i.e block) which is part of a piece
#[derive(Component)]
pub struct Mino;

/// Command to add the necessary components to draw a _Mino_ to the given entity.
pub struct SpawnMino(pub Pos, pub Option<Color>);

impl EntityCommand for SpawnMino {
    fn apply(self, entity: Entity, world: &mut World) {
        world.run_system_once_with((entity, self), spawn_mino)
    }
}

fn spawn_mino(In((entity, config)): In<(Entity, SpawnMino)>, mut commands: Commands) {
    let SpawnMino(pos, color) = config;

    let shape_config = if let Some(color) = color {
        ShapeConfig {
            color,
            corner_radii: Vec4::splat(0.1),
            ..ShapeConfig::default_2d()
        }
    } else {
        ShapeConfig {
            color: AQUA.into(),
            corner_radii: Vec4::splat(0.1),
            hollow: true,
            thickness: 1.0 / 20.0,
            thickness_type: ThicknessType::Pixels,
            ..ShapeConfig::default_2d()
        }
    };

    commands
        .entity(entity)
        .insert((Mino, SpatialBundle::from_transform(pos.into())))
        .with_shape_children(&shape_config, |shapes| {
            shapes.translate(Vec3::new(0.5, 0.5, 0.0));
            // block
            shapes.rect(Vec2::splat(0.9));
            // outline
            shapes.origin = Some(Vec3::Z * 0.1);
            shapes.hollow = true;
            shapes.thickness = 1.0 / 20.0;
            shapes.thickness_type = ThicknessType::Pixels;
            shapes.color = shapes.color.darker(0.5);
            shapes.rect(Vec2::splat(0.9));
        });
}
