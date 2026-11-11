//! How bright the neon elements of the game are. Colors brighter than 1.0 bloom.

use bevy::prelude::*;

use super::{
    Block,
    spawners::{
        matrix::MatrixFrame,
        piece::{GhostPiece, Piece},
    },
};
use crate::{
    AppSystems,
    model::{Tetrimino, TetriminoKind},
    screen::Screen,
};

/// Neon cyan, the same for all ghost pieces
const GHOST_COLOR: LinearRgba = LinearRgba::new(0.1, 1.0, 0.9, 0.8);
/// Neon blue
const FRAME_COLOR: LinearRgba = LinearRgba::rgb(0.1, 0.4, 1.0);

pub fn plugin(app: &mut App) {
    app.init_resource::<Intensities>().add_systems(
        Update,
        apply_intensities
            .in_set(AppSystems::Update)
            .run_if(resource_changed::<Intensities>)
            .run_if(in_state(Screen::Gameplay)),
    );
}

/// How much brighter than their base colors the elements of the game are drawn
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Intensities {
    /// Tetriminos in play, and in the Next and Hold Queues
    pub neon: f32,
    /// Blocks locked in the Matrix: a little brighter, so that dark colors stand out against the
    /// background, but much less than tetriminos in play
    pub locked: f32,
    /// The ghost piece
    pub ghost: f32,
    /// The frame around the Matrix
    pub frame: f32,
}

impl Default for Intensities {
    fn default() -> Self {
        Self {
            neon: 25.0,
            locked: 1.4,
            ghost: 2.0,
            frame: 25.0,
        }
    }
}

impl Intensities {
    /// The color of a tetrimino of kind `kind` while in play or in a queue
    pub fn piece_color(&self, kind: TetriminoKind) -> Color {
        kind.color(self.neon)
    }

    /// The color of the blocks a tetrimino of kind `kind` leaves in the Matrix
    pub fn locked_color(&self, kind: TetriminoKind) -> Color {
        kind.color(self.locked)
    }

    pub fn ghost_color(&self) -> Color {
        brighten(GHOST_COLOR, self.ghost)
    }

    pub fn frame_color(&self) -> Color {
        brighten(FRAME_COLOR, self.frame)
    }
}

/// `color` with its brightness, but not its alpha, multiplied by `intensity`
fn brighten(color: LinearRgba, intensity: f32) -> Color {
    (color * intensity).with_alpha(color.alpha).into()
}

/// Recolor what is already on screen when the intensities change. Alphas are kept, e.g. for the
/// dimmed held piece and blocks fading out of the Matrix, except for the ghost piece's, which is
/// part of its color.
fn apply_intensities(
    intensities: Res<Intensities>,
    pieces: Query<(&Tetrimino, &Children, Has<GhostPiece>), With<Piece>>,
    blocks: Query<(Entity, &Block)>,
    frame: Query<Entity, With<MatrixFrame>>,
    mut sprites: Query<&mut Sprite>,
) {
    for (tetrimino, children, is_ghost) in &pieces {
        for child in children.iter() {
            let Ok(mut sprite) = sprites.get_mut(child) else {
                continue;
            };
            sprite.color = if is_ghost {
                intensities.ghost_color()
            } else {
                intensities
                    .piece_color(tetrimino.kind)
                    .with_alpha(sprite.color.alpha())
            };
        }
    }

    let colors = blocks
        .iter()
        .map(|(entity, Block(kind))| (entity, intensities.locked_color(*kind)))
        .chain(
            frame
                .iter()
                .map(|entity| (entity, intensities.frame_color())),
        );
    for (entity, color) in colors {
        if let Ok(mut sprite) = sprites.get_mut(entity) {
            let alpha = sprite.color.alpha();
            sprite.color = color.with_alpha(alpha);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    /// A piece of kind T made of a single mino of color `color`, returning the mino
    fn spawn_piece(world: &mut World, color: Color, ghost: bool) -> Entity {
        let piece = world.spawn((Piece, Tetrimino::from(TetriminoKind::T))).id();
        if ghost {
            world.entity_mut(piece).insert(GhostPiece);
        }
        world
            .spawn((Sprite::from_color(color, Vec2::ONE), ChildOf(piece)))
            .id()
    }

    fn color(world: &World, entity: Entity) -> Color {
        world.get::<Sprite>(entity).unwrap().color
    }

    #[test]
    fn changing_intensities_recolors_everything_on_screen() {
        let mut world = World::new();
        let intensities = Intensities {
            neon: 4.0,
            locked: 0.5,
            ghost: 3.0,
            frame: 6.0,
        };
        world.insert_resource(intensities);
        let dimmed = spawn_piece(&mut world, Color::WHITE.with_alpha(0.3), false);
        let ghost = spawn_piece(&mut world, Color::WHITE, true);
        let fading = Color::WHITE.with_alpha(0.5);
        let block = world
            .spawn((
                Block(TetriminoKind::S),
                Sprite::from_color(fading, Vec2::ONE),
            ))
            .id();
        let frame = world
            .spawn((MatrixFrame, Sprite::from_color(Color::WHITE, Vec2::ONE)))
            .id();

        world.run_system_once(apply_intensities).unwrap();

        assert_eq!(
            color(&world, dimmed),
            TetriminoKind::T.color(4.0).with_alpha(0.3)
        );
        assert_eq!(color(&world, ghost), brighten(GHOST_COLOR, 3.0));
        assert_eq!(
            color(&world, block),
            TetriminoKind::S.color(0.5).with_alpha(0.5)
        );
        assert_eq!(color(&world, frame), brighten(FRAME_COLOR, 6.0));
    }
}
