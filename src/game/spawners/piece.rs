use bevy::{
    color::palettes::basic::AQUA,
    ecs::system::{EntityCommand, RunSystemOnce},
    prelude::*,
    sprite::Anchor,
};

use crate::model::{Pos, Tetrimino};

use super::{INITIAL_POS, Positioned};

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
        world
            .run_system_once_with(spawn, self)
            .expect("Failed to spawn Piece");
    }
}

fn spawn(In(config): In<SpawnPiece>, mut commands: Commands) {
    info!("Spawning piece");
    let SpawnPiece(parent, piece, pos, piece_type) = config;

    // Spawn a PieceBundle as a child of the given parent entity
    commands.entity(parent).with_children(|children| {
        let mut builder = children.spawn((
            Piece,
            Transform::from_xyz(pos.x as f32, pos.y as f32, 1.0),
            piece,
            Positioned(pos),
        ));
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
                c.spawn_empty().queue(SpawnMino(*p, color));
            }
        });
    });
}

#[derive(Component)]
#[require(Tetrimino, Positioned, Transform, Visibility)]
pub struct Piece;

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
    fn apply(self, mut entity_world: EntityWorldMut) {
        let entity = entity_world.id();
        entity_world
            .world_scope(|world| world.run_system_once_with(spawn_mino, (entity, self)))
            .unwrap()
    }
}

fn spawn_mino(In((entity, config)): In<(Entity, SpawnMino)>, mut commands: Commands) {
    let SpawnMino(pos, color) = config;

    commands.entity(entity).insert((
        Mino,
        Sprite::from_color(color.unwrap_or(AQUA.into()), Vec2::splat(1.0)),
        Anchor::BOTTOM_LEFT,
        Transform::from(pos),
    ));
}
