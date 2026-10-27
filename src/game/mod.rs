use bevy::{
    color::palettes::css::{BLACK, GRAY},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use input::{AutoRepeat, Direction, HardDrop, Hold, Restart, RotateLeft, RotateRight, SoftDrop};
use score::{Score, ScoreAction, ScoreEvent, TSpin};
use spawners::{
    INITIAL_POS, Positioned,
    hold_zone::{HoldQueueZone, hold_zone},
    matrix::matrix,
    next_zone::{NextTetriminoZone, next_zone},
    piece::{
        CurrentPiece, GhostPiece, HELD_UNAVAILABLE_ALPHA, HeldPiece, Mino, current_piece,
        ghost_piece, held_piece, mino, next_piece,
    },
};
use timers::Timers;

use self::matrix::Matrix;
use crate::{
    AppSystems,
    model::{Bag, Rotation, Tetrimino, TetriminoKind},
    screen::Screen,
};

#[cfg(feature = "dev")]
mod debug;
mod input;
mod matrix;
mod score;
pub mod spawners;
mod timers;
pub mod ui;

pub const MATRIX_WIDTH: u8 = 10;
pub const MATRIX_HEIGHT: u8 = 40;
pub const SCALE: f32 = 20.0;
/// The first row above the Skyline, i.e. in the Buffer Zone
pub const SKYLINE: i8 = 20;

pub fn plugin(app: &mut App) {
    app.add_sub_state::<Phase>()
        .init_resource::<GameState>()
        .insert_resource(ClearColor(BLACK.into()))
        .add_systems(
            OnEnter(Screen::Gameplay),
            (load_resources, game_setup).chain(),
        )
        // Lock down, Pattern and Eliminate take no game time, so they run back to back in the
        // frame where the piece locks.
        .add_systems(
            OnTransition {
                exited: Phase::Falling,
                entered: Phase::Generation,
            },
            (lock_down, clear_lines).chain(),
        )
        // Holding re-enters the Falling phase, so that the new piece starts falling like a newly
        // generated one
        .add_systems(
            OnTransition {
                exited: Phase::Falling,
                entered: Phase::Falling,
            },
            hold_piece,
        )
        .add_systems(OnEnter(Phase::Generation), start_generation_timer)
        .add_systems(OnEnter(Phase::Falling), start_fall_timer)
        .add_systems(
            Update,
            (
                tick_timers.in_set(AppSystems::TickTimers),
                (
                    input::auto_repeat,
                    handle_input.run_if(in_state(Phase::Falling)),
                )
                    .chain()
                    .in_set(AppSystems::RecordInput),
                restart
                    .in_set(AppSystems::RecordInput)
                    .run_if(in_state(Phase::GameOver)),
                generate_piece
                    .in_set(AppSystems::Update)
                    .run_if(in_state(Phase::Generation))
                    .run_if(generation_delay_elapsed),
            )
                .run_if(in_state(Screen::Gameplay)),
        )
        .add_systems(
            Update,
            (
                update_ghost.run_if(in_state(Phase::Falling)),
                update_mino_offsets,
                update_transforms,
                update_held_piece_color.run_if(resource_changed::<GameState>),
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
    /// Waiting to spawn the next piece
    #[default]
    Generation,
    /// The player controls the piece. This also covers the guideline's Lock phase, i.e. when the
    /// piece is on a surface and the lock down timer is running.
    Falling,
    /// The piece locked down entirely above the Skyline (Lock out), or the next piece couldn't be
    /// generated (Block out)
    GameOver,
}

#[derive(Default, Resource, Reflect)]
pub struct GameState {
    pub matrix: Matrix,
    pub bag: Bag,
    pub is_soft_dropping: bool,
    /// The piece in the Hold Queue
    pub held: Option<TetriminoKind>,
    /// Whether the player has used Hold since the last lock down. A piece must lock down between
    /// Holds.
    pub has_held: bool,
    /// The Rotation Point (1 to 5) used if the last successful move of the piece in play was a
    /// rotation, which is required for a T-Spin
    pub last_rotation_point: Option<u8>,
    /// The T-Spin performed by the piece that just locked down, scored once its Line Clears are
    /// known
    pub t_spin: Option<TSpin>,
}

/// A static block that has been committed to the matrix.
#[derive(Default, Clone, Component)]
pub struct Block;

fn load_resources(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.insert_resource(GameAssets {
        impact: asset_server.load("audio/impactPlank_medium_004.ogg"),
    });
}

fn game_setup(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut event_writer: MessageWriter<ScoreEvent>,
) {
    commands.init_resource::<Timers>();

    state.matrix.root_entity = commands.spawn_scene(matrix()).id();
    commands.spawn_scene(next_zone());
    commands.spawn_scene(hold_zone());
    input::bindings(&mut commands);

    event_writer.write(ScoreEvent::LevelStart(1));
}

fn game_cleanup(mut commands: Commands, mut state: ResMut<GameState>) {
    commands.remove_resource::<Timers>();
    // The matrix's entities are despawned along with the Gameplay screen
    *state = GameState::default();
}

fn generate_piece(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut next_phase: ResMut<NextState<Phase>>,
    next_zone: Query<Entity, With<NextTetriminoZone>>,
) -> Result {
    let next_zone_entity = next_zone.single()?;
    let tetrimino = pop_next_piece(&mut commands, &mut state.bag, next_zone_entity);

    info!("Generating new tetrimino {:?}", tetrimino.kind);

    if spawn_piece(&mut commands, &state.matrix, tetrimino) {
        next_phase.set(Phase::Falling);
    } else {
        next_phase.set(Phase::GameOver);
    }

    Ok(())
}

/// Take the next piece out of the bag, and show the one after it in the Next Queue.
fn pop_next_piece(commands: &mut Commands, bag: &mut Bag, next_zone: Entity) -> Tetrimino {
    let tetrimino = bag.pop_next().into();
    let next: Tetrimino = bag.peek_next().into();
    commands.entity(next_zone).despawn_related::<Children>();
    commands.spawn_scene(bsn! { @next_piece(next) ChildOf(next_zone) });
    tetrimino
}

/// Spawn `tetrimino` at its generation position, along with its ghost piece. Returns `false` if
/// an existing block is in the way, i.e. on a Block out.
fn spawn_piece(commands: &mut Commands, matrix: &Matrix, tetrimino: Tetrimino) -> bool {
    if !matrix.is_pos_valid(&tetrimino, &INITIAL_POS) {
        info!("Block out!");
        return false;
    }

    // The piece drops one row as soon as it is generated, unless something is in its way
    let mut pos = INITIAL_POS;
    if matrix.is_pos_valid(&tetrimino, &pos.down()) {
        pos = pos.down();
    }
    let root = matrix.root_entity;
    commands.spawn_scene(bsn! { @current_piece(tetrimino, pos) ChildOf(root) });
    let ghost_pos = matrix.lowest_valid_pos(&tetrimino, &pos);
    commands.spawn_scene(bsn! { @ghost_piece(tetrimino, ghost_pos) ChildOf(root) });

    true
}

/// Hold: put the current piece in the Hold Queue, and spawn the previously held piece in its place,
/// or the next piece if nothing was held.
fn hold_piece(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut next_phase: ResMut<NextState<Phase>>,
    current_piece: Query<(Entity, &Tetrimino), With<CurrentPiece>>,
    ghost_piece: Query<Entity, With<GhostPiece>>,
    hold_zone: Query<Entity, With<HoldQueueZone>>,
    next_zone: Query<Entity, With<NextTetriminoZone>>,
) -> Result {
    let (entity, current) = current_piece.single()?;
    let hold_zone_entity = hold_zone.single()?;
    let next_zone_entity = next_zone.single()?;
    info!("Holding tetrimino {:?}", current.kind);

    commands.entity(entity).despawn();
    for ghost in &ghost_piece {
        commands.entity(ghost).despawn();
    }

    // Held pieces are shown, and come back out, north facing
    let held: Tetrimino = current.kind.into();
    commands
        .entity(hold_zone_entity)
        .despawn_related::<Children>();
    commands.spawn_scene(bsn! { @held_piece(held) ChildOf(hold_zone_entity) });

    let tetrimino = match state.held.replace(current.kind) {
        Some(kind) => kind.into(),
        None => pop_next_piece(&mut commands, &mut state.bag, next_zone_entity),
    };
    state.has_held = true;
    state.last_rotation_point = None;

    if !spawn_piece(&mut commands, &state.matrix, tetrimino) {
        next_phase.set(Phase::GameOver);
    }

    Ok(())
}

fn start_generation_timer(mut timers: ResMut<Timers>) {
    timers.generation.reset();
}

/// Bevy evaluates all of a system's run conditions, so this still runs after leaving the Gameplay
/// screen, when the timers have been removed
fn generation_delay_elapsed(timers: Option<Res<Timers>>) -> bool {
    timers.is_some_and(|timers| timers.generation.just_finished())
}

fn start_fall_timer(
    mut timers: ResMut<Timers>,
    mut state: ResMut<GameState>,
    score: Res<Score>,
    current_piece: Single<&Positioned, With<CurrentPiece>>,
) {
    info!("Starting fall timer");
    // Levels only go up when lines are cleared, which happens between pieces
    timers.fall.set_level(score.level());
    // `handle_input` switches back to soft drop if the button is still held
    timers.fall.normal_drop();
    state.is_soft_dropping = false;
    // The fall timer is paused while a piece is on a surface, so it is still paused if the previous
    // piece landed before locking down
    timers.fall.unpause();
    // Timers keep ticking during the Generation phase, so make sure the previous piece's lock
    // timer doesn't carry over to the new piece
    timers.lock.start(current_piece.y);
}

fn tick_timers(mut timers: ResMut<Timers>, time: Res<Time>) {
    timers.tick(time.delta());
}

fn handle_input(
    mut current_piece_query: Query<(&mut Tetrimino, &mut Positioned), With<CurrentPiece>>,
    mut state: ResMut<GameState>,
    auto_repeat: Res<AutoRepeat>,
    rotate_left: Single<&ActionEvents, With<Action<RotateLeft>>>,
    rotate_right: Single<&ActionEvents, With<Action<RotateRight>>>,
    hard_drop: Single<&ActionEvents, With<Action<HardDrop>>>,
    soft_drop: Single<&ActionEvents, With<Action<SoftDrop>>>,
    hold: Single<&ActionEvents, With<Action<Hold>>>,
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

    if hold.contains(ActionEvents::FIRE) && !state.has_held {
        // See the `Falling -> Falling` transition
        next_phase.set(Phase::Falling);
        return Ok(());
    }

    // Soft drop continues to the next piece for as long as the button is held, so follow whether
    // it is held rather than when it is pressed or released
    let soft_drop_held = soft_drop.contains(ActionEvents::FIRE);
    if soft_drop_held != state.is_soft_dropping {
        state.is_soft_dropping = soft_drop_held;
        if soft_drop_held {
            timers.fall.soft_drop();
        } else {
            timers.fall.normal_drop();
        }
    }

    if timers.lock.is_paused() {
        let mut lines_soft_dropped = 0;
        for _ in 0..timers.fall.times_finished_this_tick() {
            let down_pos = pos.down();
            if state.matrix.is_pos_valid(&current_piece, &down_pos) {
                if state.is_soft_dropping {
                    lines_soft_dropped += 1;
                }
                **pos = down_pos;
                state.last_rotation_point = None;
            }
        }
        if lines_soft_dropped > 0 {
            score_messages.write(ScoreEvent::SoftDrop(lines_soft_dropped));
        }
    }

    for (rotate, rotation) in [
        (*rotate_left, Rotation::CounterClockwise),
        (*rotate_right, Rotation::Clockwise),
    ] {
        if !rotate.contains(ActionEvents::FIRE) {
            continue;
        }
        if let Some((rotated, new_pos, point)) = state.matrix.rotate(&current_piece, &pos, rotation)
        {
            *current_piece = rotated;
            **pos = new_pos;
            state.last_rotation_point = Some(point);
            timers.lock.on_move();
        }
    }

    if let Some(direction) = auto_repeat.movement() {
        let new_pos = match direction {
            Direction::Left => pos.left(),
            Direction::Right => pos.right(),
        };
        if current_piece.min_x(&new_pos) >= 0
            && current_piece.max_x(&new_pos) <= 9
            && state.matrix.is_pos_valid(&current_piece, &new_pos)
        {
            **pos = new_pos;
            state.last_rotation_point = None;
            timers.lock.on_move();
        }
    }
    if hard_drop.contains(ActionEvents::FIRE) {
        let previous_pos = **pos;
        **pos = state.matrix.lowest_valid_pos(&current_piece, &pos);
        let n = (previous_pos.y - pos.y) as u8;
        if n > 0 {
            state.last_rotation_point = None;
        }
        score_messages.write(ScoreEvent::HardDrop(n));
        // A hard dropped piece locks down immediately
        next_phase.set(Phase::Generation);
        return Ok(());
    }
    timers.lock.on_fall(pos.y);

    if state.matrix.is_on_surface(&current_piece, &pos) {
        if timers.lock.moves_exhausted() {
            info!("No moves left, locking down");
            next_phase.set(Phase::Generation);
            return Ok(());
        }
        // If we just landed on a surface, (re)start the lock timer. It isn't reset here, so a
        // piece that was lifted off a surface and lands again carries on where it left off.
        if timers.lock.is_paused() {
            info!("Starting lock timer!");
            timers.fall.pause();
            timers.lock.unpause();
        }
    } else if !timers.lock.is_paused() {
        // If we were in lock phase but are free to fall, go back to "falling" phase
        timers.lock.pause();
        timers.fall.reset();
        timers.fall.unpause();
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

/// Dim the held piece while Hold is unavailable, i.e. until the current piece locks down.
fn update_held_piece_color(
    state: Res<GameState>,
    held_piece: Query<(&Tetrimino, &Children), With<HeldPiece>>,
    mut minos: Query<&mut Sprite, With<Mino>>,
) {
    for (piece, children) in &held_piece {
        let color = if state.has_held {
            piece.kind.color().with_alpha(HELD_UNAVAILABLE_ALPHA)
        } else {
            piece.kind.color()
        };
        for child in children.iter() {
            if let Ok(mut sprite) = minos.get_mut(child) {
                sprite.color = color;
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
    mut state: ResMut<GameState>,
    assets: Res<GameAssets>,
    current_piece: Query<(Entity, &Positioned, &Tetrimino), With<CurrentPiece>>,
    ghost_piece: Query<Entity, With<GhostPiece>>,
    mut next_phase: ResMut<NextState<Phase>>,
) -> Result {
    let (entity, piece_pos, piece) = current_piece.single()?;
    info!("Locking piece");

    // Recognise T-Spins before the piece becomes part of the Matrix
    state.t_spin = state
        .last_rotation_point
        .and_then(|point| state.matrix.t_spin(piece, piece_pos, point));
    state.last_rotation_point = None;
    commands.spawn((
        AudioPlayer(assets.impact.clone()),
        PlaybackSettings::DESPAWN,
    ));

    for block_pos in piece.block_positions(piece_pos) {
        let root = state.matrix.root_entity;
        let block = commands
            .spawn_scene(bsn! {
                Block
                Positioned(block_pos)
                ChildOf(root)
                @mino(block_pos, GRAY.into())
            })
            .id();
        state.matrix.insert(block_pos, block);
    }

    commands.entity(entity).despawn();
    for ghost in &ghost_piece {
        commands.entity(ghost).despawn();
    }
    state.has_held = false;

    if piece.min_y(piece_pos) >= SKYLINE {
        info!("Lock out!");
        next_phase.set(Phase::GameOver);
    }

    Ok(())
}

/// Wait for the player to start a new game.
fn restart(
    mut commands: Commands,
    restart: Single<&ActionEvents, With<Action<Restart>>>,
    blocks: Query<Entity, With<Block>>,
    hold_zone: Query<Entity, With<HoldQueueZone>>,
    mut state: ResMut<GameState>,
    mut next_phase: ResMut<NextState<Phase>>,
    mut score_messages: MessageWriter<ScoreEvent>,
) {
    if !restart.contains(ActionEvents::FIRE) {
        return;
    }

    for block in &blocks {
        commands.entity(block).despawn();
    }
    for zone in &hold_zone {
        commands.entity(zone).despawn_related::<Children>();
    }
    let root_entity = state.matrix.root_entity;
    *state = GameState::default();
    state.matrix.root_entity = root_entity;

    score_messages.write(ScoreEvent::LevelStart(1));
    next_phase.set(Phase::Generation);
}

/// Pattern and Eliminate phases: remove full lines and collapse the blocks above them, then score
/// the lock down.
fn clear_lines(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut blocks: Query<&mut Positioned, With<Block>>,
    mut event_writer: MessageWriter<ScoreEvent>,
) {
    let lines = state.matrix.full_lines();
    let t_spin = state.t_spin.take();
    match ScoreAction::new(lines.len(), t_spin) {
        Some(action) => {
            info!("Scoring {action:?}");
            event_writer.write(ScoreEvent::LockDown(action));
        }
        None => warn!(
            "How did we complete {} lines with {t_spin:?}?!?",
            lines.len()
        ),
    }
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
}

#[cfg(test)]
mod tests {
    use bevy::ecs::{message::Messages, system::RunSystemOnce};

    use super::*;
    use crate::model::{Pos, TetriminoKind};

    fn spawn_block(world: &mut World, pos: Pos) -> Entity {
        let block = world.spawn_empty().id();
        spawn_block_entity(world, pos, block);
        block
    }

    /// Turn `entity` into a block at `pos` and add it to the matrix
    fn spawn_block_entity(world: &mut World, pos: Pos, entity: Entity) {
        world.entity_mut(entity).insert((Block, Positioned(pos)));
        world.resource_mut::<GameState>().matrix.insert(pos, entity);
    }

    #[test]
    fn clear_lines_removes_full_line_and_collapses_blocks_above() {
        let mut world = World::new();
        world.init_resource::<GameState>();
        world.init_resource::<Messages<ScoreEvent>>();

        let full_line: Vec<Entity> = (0..MATRIX_WIDTH as i8)
            .map(|x| spawn_block(&mut world, Pos::new(x, 0)))
            .collect();
        let above = spawn_block(&mut world, Pos::new(3, 1));

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
        assert_eq!(messages, [ScoreEvent::LockDown(ScoreAction::Single)]);
    }

    fn is_game_over(world: &World) -> bool {
        matches!(
            world.resource::<NextState<Phase>>(),
            NextState::Pending(Phase::GameOver)
        )
    }

    /// An app that can spawn scenes, which needs the asset server
    fn scene_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            TaskPoolPlugin::default(),
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
        ));
        app
    }

    /// An app with `tetrimino` in play at `pos`
    fn piece_app(tetrimino: Tetrimino, pos: Pos) -> App {
        let mut app = scene_app();
        let world = app.world_mut();
        world.init_resource::<GameState>();
        world.init_resource::<NextState<Phase>>();
        world.insert_resource(GameAssets {
            impact: Handle::default(),
        });
        world.spawn(NextTetriminoZone);
        world.spawn(HoldQueueZone);
        let root = world.spawn_empty().id();
        world.resource_mut::<GameState>().matrix.root_entity = root;
        world.spawn((CurrentPiece, tetrimino, Positioned(pos)));
        app
    }

    fn lock_piece(tetrimino: Tetrimino, pos: Pos) -> App {
        let mut app = piece_app(tetrimino, pos);
        let result: Result = app.world_mut().run_system_once(lock_down).unwrap();
        result.unwrap();
        app
    }

    fn hold(app: &mut App) {
        let result: Result = app.world_mut().run_system_once(hold_piece).unwrap();
        result.unwrap();
    }

    fn current_piece(world: &mut World) -> Tetrimino {
        *world
            .query_filtered::<&Tetrimino, With<CurrentPiece>>()
            .single(world)
            .unwrap()
    }

    #[test]
    fn first_hold_brings_in_the_next_piece() {
        let mut app = piece_app(TetriminoKind::T.into(), Pos::new(4, 5));
        let next = app.world().resource::<GameState>().bag.peek_next();

        hold(&mut app);

        let world = app.world_mut();
        assert_eq!(current_piece(world), next.into());
        let state = world.resource::<GameState>();
        assert_eq!(state.held, Some(TetriminoKind::T));
        assert!(state.has_held);
    }

    #[test]
    fn hold_swaps_with_the_held_piece_north_facing() {
        let mut app = piece_app(
            Tetrimino::from(TetriminoKind::T).rotated_cw(),
            Pos::new(4, 5),
        );
        app.world_mut().resource_mut::<GameState>().held = Some(TetriminoKind::I);

        hold(&mut app);

        let world = app.world_mut();
        assert_eq!(current_piece(world), TetriminoKind::I.into());
        let held_pieces: Vec<_> = world
            .query::<(&Tetrimino, &ChildOf)>()
            .iter(world)
            .filter(|(_, child_of)| world.get::<HoldQueueZone>(child_of.parent()).is_some())
            .map(|(tetrimino, _)| *tetrimino)
            .collect();
        assert_eq!(held_pieces, [TetriminoKind::T.into()]);
        assert_eq!(world.resource::<GameState>().held, Some(TetriminoKind::T));
    }

    #[test]
    fn held_piece_blocked_at_generation_is_block_out() {
        let mut app = piece_app(TetriminoKind::T.into(), Pos::new(4, 5));
        let world = app.world_mut();
        world.resource_mut::<GameState>().held = Some(TetriminoKind::I);
        for x in 0..MATRIX_WIDTH as i8 {
            spawn_block(world, Pos::new(x, INITIAL_POS.y));
        }

        hold(&mut app);

        assert!(is_game_over(app.world()));
    }

    #[test]
    fn locking_down_allows_holding_again() {
        let mut app = piece_app(TetriminoKind::T.into(), Pos::new(4, 0));
        app.world_mut().resource_mut::<GameState>().has_held = true;

        let result: Result = app.world_mut().run_system_once(lock_down).unwrap();
        result.unwrap();

        assert!(!app.world().resource::<GameState>().has_held);
    }

    fn held_mino_colors(app: &mut App) -> Vec<Color> {
        app.world_mut()
            .run_system_once(update_held_piece_color)
            .unwrap();
        let world = app.world_mut();
        world
            .query::<(&Sprite, &ChildOf)>()
            .iter(world)
            .filter(|(_, child_of)| world.get::<HeldPiece>(child_of.parent()).is_some())
            .map(|(sprite, _)| sprite.color)
            .collect()
    }

    #[test]
    fn held_piece_is_dimmed_until_lock_down() {
        let mut app = piece_app(TetriminoKind::T.into(), Pos::new(4, 5));

        hold(&mut app);
        let dimmed = TetriminoKind::T.color().with_alpha(HELD_UNAVAILABLE_ALPHA);
        assert_eq!(held_mino_colors(&mut app), [dimmed; 4]);

        let result: Result = app.world_mut().run_system_once(lock_down).unwrap();
        result.unwrap();
        assert_eq!(held_mino_colors(&mut app), [TetriminoKind::T.color(); 4]);
    }

    #[test]
    fn leaving_gameplay_cleans_up_the_matrix() {
        let mut app = scene_app();
        app.add_plugins(bevy::state::app::StatesPlugin)
            .insert_state(Screen::Gameplay)
            .init_resource::<GameState>()
            .add_systems(OnExit(Screen::Gameplay), game_cleanup);
        app.update();

        let world = app.world_mut();
        let root = world.spawn_scene(matrix()).unwrap().id();
        world.resource_mut::<GameState>().matrix.root_entity = root;
        let block = world.spawn(ChildOf(root)).id();
        spawn_block_entity(world, Pos::new(0, 0), block);

        world
            .resource_mut::<NextState<Screen>>()
            .set(Screen::Splash);
        app.update();

        let world = app.world();
        assert!(
            world.get_entity(root).is_err(),
            "matrix should be despawned"
        );
        assert!(
            world.get_entity(block).is_err(),
            "block should be despawned"
        );
        let matrix = &world.resource::<GameState>().matrix;
        assert_eq!(matrix.root_entity, Entity::PLACEHOLDER);
        assert_eq!(matrix.iter_non_empty().count(), 0);
    }

    /// Lock down a south facing T into a T-Slot that completes the two bottom lines, and return
    /// the resulting score event
    fn lock_t_into_t_slot(last_rotation_point: Option<u8>) -> Vec<ScoreEvent> {
        let mut app = piece_app(
            Tetrimino::from(TetriminoKind::T).rotated_cw().rotated_cw(),
            Pos::new(1, 1),
        );
        let world = app.world_mut();
        world.init_resource::<Messages<ScoreEvent>>();
        world.resource_mut::<GameState>().last_rotation_point = last_rotation_point;
        for x in (0..MATRIX_WIDTH as i8).filter(|&x| x != 1) {
            spawn_block(world, Pos::new(x, 0));
        }
        for x in 3..MATRIX_WIDTH as i8 {
            spawn_block(world, Pos::new(x, 1));
        }
        // Overhang over the T-Slot
        spawn_block(world, Pos::new(2, 2));

        let result: Result = world.run_system_once(lock_down).unwrap();
        result.unwrap();
        world.run_system_once(clear_lines).unwrap();

        world
            .resource::<Messages<ScoreEvent>>()
            .iter_current_update_messages()
            .copied()
            .collect()
    }

    #[test]
    fn rotating_into_a_t_slot_is_a_t_spin() {
        assert_eq!(
            lock_t_into_t_slot(Some(1)),
            [ScoreEvent::LockDown(ScoreAction::TSpinDouble)]
        );
    }

    #[test]
    fn moving_into_a_t_slot_is_not_a_t_spin() {
        assert_eq!(
            lock_t_into_t_slot(None),
            [ScoreEvent::LockDown(ScoreAction::Double)]
        );
    }

    #[test]
    fn locking_without_clearing_lines_is_scored() {
        let mut app = piece_app(TetriminoKind::T.into(), Pos::new(4, 0));
        let world = app.world_mut();
        world.init_resource::<Messages<ScoreEvent>>();

        let result: Result = world.run_system_once(lock_down).unwrap();
        result.unwrap();
        world.run_system_once(clear_lines).unwrap();

        let messages: Vec<_> = world
            .resource::<Messages<ScoreEvent>>()
            .iter_current_update_messages()
            .copied()
            .collect();
        assert_eq!(messages, [ScoreEvent::LockDown(ScoreAction::Lock)]);
    }

    #[test]
    fn locking_adds_blocks_to_the_matrix() {
        let tetrimino = Tetrimino::from(TetriminoKind::T);
        let pos = Pos::new(4, 0);
        let app = lock_piece(tetrimino, pos);
        let world = app.world();

        let matrix = &world.resource::<GameState>().matrix;
        for block_pos in tetrimino.block_positions(&pos) {
            let block = matrix.at_pos(block_pos);
            assert_eq!(world.get::<Positioned>(block).unwrap().0, block_pos);
        }
        assert_eq!(matrix.iter_non_empty().count(), 4);
    }

    #[test]
    fn locking_entirely_above_skyline_is_lock_out() {
        let app = lock_piece(TetriminoKind::I.into(), Pos::new(4, SKYLINE));
        assert!(is_game_over(app.world()));
    }

    #[test]
    fn locking_partially_above_skyline_is_not_game_over() {
        // The O piece covers this row and the one above
        let app = lock_piece(TetriminoKind::O.into(), Pos::new(4, SKYLINE - 1));
        assert!(!is_game_over(app.world()));
    }

    #[test]
    fn blocked_spawn_position_is_block_out() {
        let mut app = scene_app();
        let world = app.world_mut();
        world.init_resource::<GameState>();
        world.init_resource::<NextState<Phase>>();
        world.spawn(NextTetriminoZone);
        for x in 0..MATRIX_WIDTH as i8 {
            spawn_block(world, Pos::new(x, INITIAL_POS.y));
        }

        let result: Result = world.run_system_once(generate_piece).unwrap();
        result.unwrap();

        assert!(is_game_over(world));
        assert!(
            world
                .query_filtered::<(), With<CurrentPiece>>()
                .iter(world)
                .next()
                .is_none()
        );
    }

    #[test]
    fn pieces_are_generated_where_the_guideline_says() {
        // Returns the (min x, max x, min y, max y) of the piece's cells, counting from 1 like the
        // guideline does
        let bounds = |kind: TetriminoKind| {
            let cells = Tetrimino::from(kind).block_positions(&INITIAL_POS);
            let xs = cells.iter().map(|p| p.x + 1);
            let ys = cells.iter().map(|p| p.y + 1);
            (
                xs.clone().min().unwrap(),
                xs.max().unwrap(),
                ys.clone().min().unwrap(),
                ys.max().unwrap(),
            )
        };

        // Three minos wide: (4, 21) to (6, 21), on the 21st and 22nd rows
        for kind in [
            TetriminoKind::T,
            TetriminoKind::L,
            TetriminoKind::J,
            TetriminoKind::S,
            TetriminoKind::Z,
        ] {
            assert_eq!(bounds(kind), (4, 6, 21, 22), "{kind:?}");
        }
        // I: (4, 21) to (7, 21)
        assert_eq!(bounds(TetriminoKind::I), (4, 7, 21, 21));
        // O: (5, 21) to (6, 22)
        assert_eq!(bounds(TetriminoKind::O), (5, 6, 21, 22));
    }

    /// An app with a piece falling, where only the soft drop button is held
    fn soft_drop_app() -> App {
        let mut app = piece_app(TetriminoKind::T.into(), Pos::new(4, 10));
        let world = app.world_mut();
        world.init_resource::<Timers>();
        world.init_resource::<Score>();
        world.init_resource::<AutoRepeat>();
        world.init_resource::<Messages<ScoreEvent>>();
        world.spawn((Action::<RotateLeft>::new(), ActionEvents::empty()));
        world.spawn((Action::<RotateRight>::new(), ActionEvents::empty()));
        world.spawn((Action::<HardDrop>::new(), ActionEvents::empty()));
        world.spawn((Action::<Hold>::new(), ActionEvents::empty()));
        world.spawn((Action::<SoftDrop>::new(), ActionEvents::FIRE));
        app
    }

    fn run_handle_input(world: &mut World) {
        let result: Result = world.run_system_once(handle_input).unwrap();
        result.unwrap();
    }

    fn score_messages(world: &mut World) -> Vec<ScoreEvent> {
        world
            .resource_mut::<Messages<ScoreEvent>>()
            .drain()
            .collect()
    }

    #[test]
    fn soft_drop_is_scored_while_the_button_is_held() {
        let mut app = soft_drop_app();
        let world = app.world_mut();

        run_handle_input(world);

        assert_eq!(score_messages(world), [ScoreEvent::SoftDrop(1)]);
    }

    #[test]
    fn soft_drop_continues_to_the_next_piece() {
        let mut app = soft_drop_app();
        let world = app.world_mut();
        run_handle_input(world);
        score_messages(world);

        // A new piece starts falling with the button still held
        world.run_system_once(start_fall_timer).unwrap();
        run_handle_input(world);

        assert_eq!(score_messages(world), [ScoreEvent::SoftDrop(1)]);
        let mut soft_drop_timer = timers::FallTimer::new();
        soft_drop_timer.soft_drop();
        let timers = world.resource::<Timers>();
        assert_eq!(timers.fall.duration(), soft_drop_timer.duration());
    }

    #[test]
    fn falling_after_releasing_soft_drop_is_not_scored() {
        let mut app = soft_drop_app();
        let world = app.world_mut();
        run_handle_input(world);
        score_messages(world);

        let mut soft_drop = world.query_filtered::<&mut ActionEvents, With<Action<SoftDrop>>>();
        *soft_drop.single_mut(world).unwrap() = ActionEvents::empty();
        run_handle_input(world);
        let fall_duration = world.resource::<Timers>().fall.duration();
        world.resource_mut::<Timers>().tick(fall_duration);
        run_handle_input(world);

        assert!(!world.resource::<GameState>().is_soft_dropping);
        assert_eq!(score_messages(world), []);
    }

    #[test]
    fn new_piece_falls_even_if_previous_one_landed() {
        let mut world = World::new();
        let mut timers = Timers::default();
        // The previous piece landed, which pauses the fall timer
        timers.fall.pause();
        world.insert_resource(timers);
        world.init_resource::<Score>();
        world.init_resource::<GameState>();
        world.spawn((CurrentPiece, Positioned(INITIAL_POS)));

        world.run_system_once(start_fall_timer).unwrap();

        assert!(!world.resource::<Timers>().fall.is_paused());
    }
}
