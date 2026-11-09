//! The Animate phase: full lines glow white, then disappear from left to right. Their blocks stay
//! in the Matrix until the animation is over (see Appendix A1.2.6 of the guideline).

use bevy::prelude::*;

use super::{
    Block, GameState, MATRIX_WIDTH, Phase, intensities::Intensities, spawners::Positioned,
    timers::Timers,
};
use crate::{AppSystems, screen::Screen};

/// Brighter than white so that the blocks bloom
const GLOW_COLOR: LinearRgba = LinearRgba::rgb(26.0, 26.0, 26.0);
/// How long the blocks take to turn white, in seconds
const GLOW_DURATION: f32 = 0.12;
/// Delay between a column starting to disappear and the next one, in seconds
const COLUMN_DELAY: f32 = 0.02;
/// How long a block takes to disappear, in seconds
const FADE_DURATION: f32 = 0.06;
/// How long the whole animation takes, in seconds
pub(super) const DURATION: f32 =
    GLOW_DURATION + (MATRIX_WIDTH as f32 - 1.0) * COLUMN_DELAY + FADE_DURATION;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Phase::Animate), start_timer)
        .add_systems(
            Update,
            animate
                .in_set(AppSystems::Update)
                .run_if(in_state(Phase::Animate))
                .run_if(in_state(Screen::Gameplay)),
        );
}

fn start_timer(mut timers: ResMut<Timers>) {
    timers.line_clear.reset();
}

/// The color of a block of color `color` in column `x`, `elapsed` seconds into the animation.
fn block_color(color: Color, elapsed: f32, x: i8) -> Color {
    let glow = (elapsed / GLOW_DURATION).min(1.0);
    let fade_start = GLOW_DURATION + f32::from(x) * COLUMN_DELAY;
    let fade = ((elapsed - fade_start) / FADE_DURATION).clamp(0.0, 1.0);
    LinearRgba::from(color)
        .mix(&GLOW_COLOR, glow)
        .with_alpha(1.0 - fade)
        .into()
}

/// Play the animation for any full lines, then move on to the next piece. Without any full lines
/// there is nothing to wait for.
fn animate(
    timers: Res<Timers>,
    state: Res<GameState>,
    intensities: Res<Intensities>,
    mut next_phase: ResMut<NextState<Phase>>,
    mut blocks: Query<(&Block, &Positioned, &mut Sprite)>,
) {
    // A Lock out in `lock_down` ends the game right away
    if matches!(*next_phase, NextState::Pending(Phase::GameOver)) {
        return;
    }

    let elapsed = timers.line_clear.elapsed_secs();
    for entity in state.matrix.entities_to_delete() {
        if let Ok((Block(kind), pos, mut sprite)) = blocks.get_mut(entity) {
            sprite.color = block_color(intensities.locked_color(*kind), elapsed, pos.x);
        }
    }

    if state.matrix.full_lines().is_empty() || timers.line_clear.is_finished() {
        // See the `Animate -> Generation` transition
        next_phase.set(Phase::Generation);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::ecs::system::RunSystemOnce;

    use super::*;
    use crate::model::{Pos, TetriminoKind};

    fn alpha(elapsed: f32, x: i8) -> f32 {
        block_color(Color::WHITE, elapsed, x).alpha()
    }

    #[test]
    fn blocks_turn_white_before_disappearing() {
        let color = Intensities::default().locked_color(TetriminoKind::T);
        assert_eq!(block_color(color, 0.0, 0), LinearRgba::from(color).into());
        assert_eq!(block_color(color, GLOW_DURATION, 0), GLOW_COLOR.into());
    }

    #[test]
    fn blocks_disappear_from_left_to_right() {
        let elapsed = GLOW_DURATION + 2.0 * COLUMN_DELAY;
        assert_eq!(alpha(elapsed + FADE_DURATION, 2), 0.0);
        assert_eq!(alpha(elapsed, 2), 1.0);
        assert_eq!(alpha(elapsed + FADE_DURATION, 9), 1.0);
    }

    #[test]
    fn the_last_column_is_gone_when_the_animation_ends() {
        assert_eq!(alpha(DURATION, MATRIX_WIDTH as i8 - 1), 0.0);
    }

    /// A world with a line at the bottom of the Matrix, which is full if `full`
    fn line_world(full: bool) -> (World, Entity) {
        let mut world = World::new();
        world.init_resource::<GameState>();
        world.init_resource::<NextState<Phase>>();
        world.init_resource::<Timers>();
        world.init_resource::<Intensities>();
        let width = MATRIX_WIDTH as i8 - i8::from(!full);
        let blocks: Vec<_> = (0..width)
            .map(|x| {
                let pos = Pos::new(x, 0);
                let block = world
                    .spawn((Block::default(), Positioned(pos), Sprite::default()))
                    .id();
                world.resource_mut::<GameState>().matrix.insert(pos, block);
                block
            })
            .collect();
        (world, blocks[0])
    }

    fn next_phase(world: &World) -> Option<Phase> {
        match world.resource::<NextState<Phase>>() {
            NextState::Pending(phase) => Some(*phase),
            _ => None,
        }
    }

    #[test]
    fn full_lines_wait_for_the_animation() {
        let (mut world, block) = line_world(true);
        let halfway = Duration::from_secs_f32(DURATION / 2.0);
        world.resource_mut::<Timers>().tick(halfway);

        world.run_system_once(animate).unwrap();

        assert_eq!(next_phase(&world), None);
        assert_eq!(
            world.get::<Sprite>(block).unwrap().color,
            block_color(
                Intensities::default().locked_color(Block::default().0),
                halfway.as_secs_f32(),
                0
            )
        );

        world.resource_mut::<Timers>().tick(halfway * 2);
        world.run_system_once(animate).unwrap();

        assert_eq!(next_phase(&world), Some(Phase::Generation));
    }

    #[test]
    fn nothing_to_animate_moves_on_immediately() {
        let (mut world, block) = line_world(false);
        let color = world.get::<Sprite>(block).unwrap().color;

        world.run_system_once(animate).unwrap();

        assert_eq!(next_phase(&world), Some(Phase::Generation));
        assert_eq!(world.get::<Sprite>(block).unwrap().color, color);
    }

    #[test]
    fn lock_out_is_not_overridden() {
        let (mut world, _) = line_world(false);
        world
            .resource_mut::<NextState<Phase>>()
            .set(Phase::GameOver);

        world.run_system_once(animate).unwrap();

        assert_eq!(next_phase(&world), Some(Phase::GameOver));
    }
}
