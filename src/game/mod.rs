use bevy::{
    color::palettes::css::{BLACK, GRAY},
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use input::{HardDrop, Left, Right, RotateLeft, RotateRight, SoftDrop};
use score::ScoreEvent;
use spawners::{
    INITIAL_POS, Positioned, SpawnMatrix, SpawnNextZone, SpawnPiece,
    next_zone::NextTetriminoZone,
    piece::{CurrentPiece, GhostPiece, Mino, SpawnMino},
};
use timers::Timers;

use self::matrix::Matrix;
use crate::{
    AppSystems,
    model::{Bag, Tetrimino},
    screen::Screen,
};

#[cfg(feature = "dev")]
mod debug;
mod input;
mod matrix;
mod score;
pub mod spawners;
mod timers;
mod ui;

pub const MATRIX_WIDTH: u8 = 10;
pub const MATRIX_HEIGHT: u8 = 40;
pub const SCALE: f32 = 20.0;

pub fn plugin(app: &mut App) {
    app.add_sub_state::<Phase>()
        .init_resource::<GameState>()
        .insert_resource(ClearColor(BLACK.into()))
        .add_systems(
            OnEnter(Screen::Gameplay),
            (load_resources, game_setup).chain(),
        )
        // Lock down, Pattern and Eliminate take no game time, so they run back to back in the
        // frame where the piece locks, right before the next piece is generated.
        .add_systems(
            OnTransition {
                exited: Phase::Falling,
                entered: Phase::Generation,
            },
            (lock_down, clear_lines).chain(),
        )
        .add_systems(
            OnEnter(Phase::Generation),
            (clean_up_pieces, generate_piece).chain(),
        )
        .add_systems(OnEnter(Phase::Falling), start_fall_timer)
        .add_systems(
            Update,
            (
                tick_timers.in_set(AppSystems::TickTimers),
                handle_input.in_set(AppSystems::RecordInput),
            )
                .run_if(in_state(Phase::Falling)),
        )
        .add_systems(
            Update,
            (
                update_ghost.run_if(in_state(Phase::Falling)),
                update_mino_offsets,
                update_transforms,
            )
                .chain()
                .in_set(AppSystems::Update)
                .run_if(in_state(Screen::Gameplay)),
        )
        .add_systems(OnExit(Screen::Gameplay), game_cleanup);

    app.add_plugins((input::plugin, spawners::plugin, score::plugin, ui::plugin));

    #[cfg(feature = "dev")]
    app.add_plugins(debug::plugin);
}

#[derive(Resource)]
pub struct GameAssets {
    impact: Handle<AudioSource>,
}

/// The phases of the Tetris engine that take up game time (see Appendix A1 of the Tetris Design
/// Guideline).
///
/// The other phases are instantaneous so they are not modeled as states: Lock down, Pattern and
/// Eliminate run on the `Falling -> Generation` transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, SubStates, Reflect, strum::Display)]
#[source(Screen = Screen::Gameplay)]
pub enum Phase {
    /// Spawning the next piece
    #[default]
    Generation,
    /// The player controls the piece. This also covers the guideline's Lock phase, i.e. when the
    /// piece is on a surface and the lock down timer is running.
    Falling,
}

#[derive(Default, Resource, Reflect)]
pub struct GameState {
    pub matrix: Matrix,
    pub bag: Bag,
    pub is_soft_dropping: bool,
    pub lines_soft_dropped: u8,
}

/// A static block that has been committed to the matrix.
#[derive(Component)]
#[component(on_add = on_block_add)]
pub struct Block;

fn on_block_add(mut world: DeferredWorld, ctx: HookContext) {
    let pos = world
        .entity(ctx.entity)
        .get::<Positioned>()
        .copied()
        .expect("Block component without Pos!");
    info!("Block was added at {}", *pos);
    let mut state = world.resource_mut::<GameState>();
    state.matrix.insert(*pos, ctx.entity);
}

fn load_resources(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(GameAssets {
        impact: asset_server.load("audio/impactPlank_medium_004.ogg"),
    });
}

fn game_setup(mut commands: Commands, mut event_writer: MessageWriter<ScoreEvent>) {
    commands.init_resource::<Timers>();

    commands.queue(SpawnMatrix);
    commands.queue(SpawnNextZone);
    input::bindings(&mut commands);

    event_writer.write(ScoreEvent::LevelStart(1));
}

fn game_cleanup(mut commands: Commands) {
    commands.remove_resource::<Timers>();
}

fn clean_up_pieces(mut commands: Commands, pieces: Query<Entity, With<Tetrimino>>) {
    for piece in pieces.into_iter() {
        info!("Despawning tetrimino");
        commands.entity(piece).despawn();
    }
}

fn generate_piece(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut next_phase: ResMut<NextState<Phase>>,
    next_zone: Query<Entity, With<NextTetriminoZone>>,
) -> Result {
    let next_zone_entity = next_zone.single()?;
    let tetrimino: Tetrimino = state.bag.pop_next().into();
    let next_piece: Tetrimino = state.bag.peek_next().into();

    info!("Generating new tetrimino {:?}", tetrimino.kind);

    // The piece drops one row as soon as it is generated, unless something is in its way
    let mut pos = INITIAL_POS;
    if state.matrix.is_pos_valid(&tetrimino, &pos.down()) {
        pos = pos.down();
    }
    commands.queue(SpawnPiece::current(tetrimino, pos).with_parent(state.matrix.root_entity));
    let ghost_pos = state.matrix.lowest_valid_pos(&tetrimino, &pos);
    commands.queue(SpawnPiece::ghost(tetrimino, ghost_pos).with_parent(state.matrix.root_entity));

    commands.queue(SpawnPiece::next(next_piece).with_parent(next_zone_entity));

    next_phase.set(Phase::Falling);

    Ok(())
}

fn start_fall_timer(mut timers: ResMut<Timers>) {
    info!("Starting fall timer");
    timers.fall.normal_drop();
}

fn tick_timers(mut timers: ResMut<Timers>, time: Res<Time>) {
    timers.tick(time.delta());
}

fn handle_input(
    mut current_piece_query: Query<(&mut Tetrimino, &mut Positioned), With<CurrentPiece>>,
    mut state: ResMut<GameState>,
    left: Single<&ActionEvents, With<Action<Left>>>,
    right: Single<&ActionEvents, With<Action<Right>>>,
    rotate_left: Single<&ActionEvents, With<Action<RotateLeft>>>,
    rotate_right: Single<&ActionEvents, With<Action<RotateRight>>>,
    hard_drop: Single<&ActionEvents, With<Action<HardDrop>>>,
    soft_drop: Single<&ActionEvents, With<Action<SoftDrop>>>,
    mut timers: ResMut<Timers>,
    mut next_phase: ResMut<NextState<Phase>>,
    mut score_messages: MessageWriter<ScoreEvent>,
) -> Result {
    let (mut current_piece, mut pos) = current_piece_query.single_mut()?;

    // If lock timer has expired -> lock down (see the `Falling -> Generation` transition)
    if timers.lock.times_finished_this_tick() > 0 {
        next_phase.set(Phase::Generation);
        return Ok(());
    }

    if timers.lock.is_paused() {
        for _ in 0..timers.fall.times_finished_this_tick() {
            let down_pos = pos.down();
            if state.matrix.is_pos_valid(&current_piece, &down_pos) {
                if state.is_soft_dropping {
                    state.lines_soft_dropped += 1;
                }
                **pos = down_pos;
            }
        }
    }

    if rotate_left.contains(ActionEvents::FIRE) {
        let rotated = current_piece.rotated_ccw();
        if state.matrix.is_pos_valid(&rotated, &pos) {
            *current_piece = rotated;
        }
    }
    if rotate_right.contains(ActionEvents::FIRE) {
        let rotated = current_piece.rotated_cw();
        if state.matrix.is_pos_valid(&rotated, &pos) {
            *current_piece = rotated;
        }
    }

    if left.contains(ActionEvents::FIRE) {
        let left_pos = pos.left();
        if current_piece.min_x(&left_pos) >= 0
            && state.matrix.is_pos_valid(&current_piece, &left_pos)
        {
            **pos = left_pos;
        }
    } else if right.contains(ActionEvents::FIRE) {
        // {
        let right_pos = pos.right();
        if current_piece.max_x(&right_pos) <= 9
            && state.matrix.is_pos_valid(&current_piece, &right_pos)
        {
            **pos = right_pos;
        }
    }
    if hard_drop.contains(ActionEvents::FIRE) {
        let previous_pos = **pos;
        **pos = state.matrix.lowest_valid_pos(&current_piece, &pos);
        let n = (previous_pos.y - pos.y) as u8;
        score_messages.write(ScoreEvent::HardDrop(n));
        // A hard dropped piece locks down immediately
        next_phase.set(Phase::Generation);
        return Ok(());
    }
    if soft_drop.contains(ActionEvents::START) {
        state.is_soft_dropping = true;
        state.lines_soft_dropped = 0;
        timers.fall.soft_drop();
    } else if soft_drop.contains(ActionEvents::COMPLETE) {
        state.is_soft_dropping = false;
        if state.lines_soft_dropped > 0 {
            score_messages.write(ScoreEvent::SoftDrop(state.lines_soft_dropped));
            state.lines_soft_dropped = 0;
        }
        timers.fall.normal_drop();
    }

    if state.matrix.is_on_surface(&current_piece, &pos) {
        // If we just landed on a surface, kick off the lock timer
        if timers.lock.is_paused() {
            info!("Starting lock timer!");
            timers.fall.pause();
            timers.lock.reset();
            timers.lock.unpause();
        }
    } else {
        // If we were in lock phase but are free to fall, go back to "falling" phase
        if !timers.lock.is_paused() {
            timers.lock.pause();
            timers.fall.normal_drop();
            timers.fall.unpause();
        }
    }

    Ok(())
}

fn update_ghost(
    current: Query<(&Positioned, &Tetrimino), (With<CurrentPiece>, Without<GhostPiece>)>,
    mut ghost: Query<(&mut Positioned, &mut Tetrimino), (With<GhostPiece>, Without<CurrentPiece>)>,
    state: Res<GameState>,
) -> Result {
    let (current_pos, current_tetrimino) = current.single()?;
    let (mut ghost_pos, mut ghost_tetrimino) = ghost.single_mut()?;
    let new_pos = state
        .matrix
        .lowest_valid_pos(current_tetrimino, &current_pos.0);
    if ghost_tetrimino.as_ref() != current_tetrimino {
        *ghost_tetrimino = *current_tetrimino;
    }
    if ghost_pos.0 != new_pos {
        ghost_pos.0 = new_pos;
    }

    Ok(())
}

/// Position each piece's minos relative to the piece when its facing changes.
fn update_mino_offsets(
    pieces: Query<(&Tetrimino, &Children), Changed<Tetrimino>>,
    mut minos: Query<&mut Transform, With<Mino>>,
) {
    for (piece, children) in &pieces {
        for (child, offset) in children.iter().zip(piece.block_offsets()) {
            if let Ok(mut transform) = minos.get_mut(child) {
                *transform = offset.into();
            }
        }
    }
}

/// Update the Transform of pieces and blocks based on their grid position.
///
/// A piece's position is the position (in grid coordinates) of its "visual center". The minos
/// that make up the piece are positioned relative to that.
fn update_transforms(mut query: Query<(&mut Transform, &Positioned), Changed<Positioned>>) {
    for (mut transform, pos) in &mut query {
        // Preserve z so the ghost stays behind the current piece
        let z = transform.translation.z;
        transform.translation = Transform::from(pos.0).translation.with_z(z);
    }
}

/// Lock down phase: turn the current piece into static blocks.
fn lock_down(
    mut commands: Commands,
    state: Res<GameState>,
    assets: Res<GameAssets>,
    current_piece: Query<(Entity, &Positioned, &Tetrimino), With<CurrentPiece>>,
    ghost_piece: Query<Entity, With<GhostPiece>>,
) -> Result {
    let (entity, piece_pos, piece) = current_piece.single()?;
    info!("Locking piece");
    commands.spawn((
        AudioPlayer(assets.impact.clone()),
        PlaybackSettings::DESPAWN,
    ));

    commands
        .entity(state.matrix.root_entity)
        .with_children(|children| {
            for block_pos in piece.block_positions(piece_pos) {
                children
                    .spawn((Block, Positioned(block_pos)))
                    .queue(SpawnMino(block_pos, GRAY.into()));
            }
        });

    commands.entity(entity).despawn();
    for ghost in &ghost_piece {
        commands.entity(ghost).despawn();
    }

    Ok(())
}

/// Pattern and Eliminate phases: remove full lines and collapse the blocks above them.
///
/// The new blocks are only added to the matrix once the commands from `lock_down` are applied, so
/// this must run after it.
fn clear_lines(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut blocks: Query<&mut Positioned, With<Block>>,
    mut event_writer: MessageWriter<ScoreEvent>,
) {
    let lines = state.matrix.full_lines();
    if lines.is_empty() {
        return;
    }

    for e in state.matrix.entities_to_delete() {
        info!("Despawning block {e}");
        commands.entity(e).despawn();
    }

    // Remove lines from the top down so the indices of the remaining lines stay valid
    for &line in lines.iter().rev() {
        info!("Removing line {line}");
        state.matrix.delete_line(line);
    }

    // Reflect new positions
    for (pos, entity) in state.matrix.iter_non_empty() {
        if let Ok(mut block_pos) = blocks.get_mut(entity) {
            if block_pos.0 != pos {
                block_pos.0 = pos;
            }
        } else {
            warn!("Missing entity {entity}");
        }
    }

    match lines.len() {
        1 => {
            event_writer.write(ScoreEvent::Single);
        }
        2 => {
            event_writer.write(ScoreEvent::Double);
        }
        3 => {
            event_writer.write(ScoreEvent::Triple);
        }
        4 => {
            event_writer.write(ScoreEvent::Tetris);
        }
        n => warn!("How did we complete {n} lines?!?"),
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::{message::Messages, system::RunSystemOnce};

    use super::*;
    use crate::model::Pos;

    #[test]
    fn clear_lines_removes_full_line_and_collapses_blocks_above() {
        let mut world = World::new();
        world.init_resource::<GameState>();
        world.init_resource::<Messages<ScoreEvent>>();

        let full_line: Vec<Entity> = (0..MATRIX_WIDTH as i8)
            .map(|x| world.spawn((Block, Positioned(Pos::new(x, 0)))).id())
            .collect();
        let above = world.spawn((Block, Positioned(Pos::new(3, 1)))).id();

        world.run_system_once(clear_lines).unwrap();

        for e in full_line {
            assert!(
                world.get_entity(e).is_err(),
                "block {e} should be despawned"
            );
        }
        assert_eq!(world.get::<Positioned>(above).unwrap().0, Pos::new(3, 0));
        let state = world.resource::<GameState>();
        assert_eq!(state.matrix.at_pos(Pos::new(3, 0)), above);
        assert_eq!(state.matrix.at_pos(Pos::new(3, 1)), Entity::PLACEHOLDER);
        let messages: Vec<_> = world
            .resource::<Messages<ScoreEvent>>()
            .iter_current_update_messages()
            .copied()
            .collect();
        assert_eq!(messages, [ScoreEvent::Single]);
    }
}
