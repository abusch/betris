use bevy::{
    color::palettes::css::{BLACK, GRAY},
    prelude::*,
};
use bevy_enhanced_input::prelude::*;
use input::{HardDrop, Left, Restart, Right, RotateLeft, RotateRight, SoftDrop};
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
        .add_systems(OnEnter(Phase::Generation), start_generation_timer)
        .add_systems(OnEnter(Phase::Falling), start_fall_timer)
        .add_systems(
            Update,
            (
                tick_timers.in_set(AppSystems::TickTimers),
                handle_input
                    .in_set(AppSystems::RecordInput)
                    .run_if(in_state(Phase::Falling)),
                restart
                    .in_set(AppSystems::RecordInput)
                    .run_if(in_state(Phase::GameOver)),
                (clean_up_pieces, generate_piece)
                    .chain()
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
    pub lines_soft_dropped: u8,
}

/// A static block that has been committed to the matrix.
#[derive(Component)]
pub struct Block;

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

fn game_cleanup(mut commands: Commands, mut state: ResMut<GameState>) {
    commands.remove_resource::<Timers>();
    // The matrix's entities are despawned along with the Gameplay screen
    *state = GameState::default();
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

    if !state.matrix.is_pos_valid(&tetrimino, &INITIAL_POS) {
        info!("Block out!");
        next_phase.set(Phase::GameOver);
        return Ok(());
    }

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
    current_piece: Single<&Positioned, With<CurrentPiece>>,
) {
    info!("Starting fall timer");
    timers.fall.normal_drop();
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
            timers.lock.on_move();
        }
    }
    if rotate_right.contains(ActionEvents::FIRE) {
        let rotated = current_piece.rotated_cw();
        if state.matrix.is_pos_valid(&rotated, &pos) {
            *current_piece = rotated;
            timers.lock.on_move();
        }
    }

    if left.contains(ActionEvents::FIRE) {
        let left_pos = pos.left();
        if current_piece.min_x(&left_pos) >= 0
            && state.matrix.is_pos_valid(&current_piece, &left_pos)
        {
            **pos = left_pos;
            timers.lock.on_move();
        }
    } else if right.contains(ActionEvents::FIRE) {
        let right_pos = pos.right();
        if current_piece.max_x(&right_pos) <= 9
            && state.matrix.is_pos_valid(&current_piece, &right_pos)
        {
            **pos = right_pos;
            timers.lock.on_move();
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
    commands.spawn((
        AudioPlayer(assets.impact.clone()),
        PlaybackSettings::DESPAWN,
    ));

    for block_pos in piece.block_positions(piece_pos) {
        let block = commands
            .spawn((
                Block,
                Positioned(block_pos),
                ChildOf(state.matrix.root_entity),
            ))
            .queue(SpawnMino(block_pos, GRAY.into()))
            .id();
        state.matrix.insert(block_pos, block);
    }

    commands.entity(entity).despawn();
    for ghost in &ghost_piece {
        commands.entity(ghost).despawn();
    }

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
    let root_entity = state.matrix.root_entity;
    *state = GameState::default();
    state.matrix.root_entity = root_entity;

    score_messages.write(ScoreEvent::LevelStart(1));
    next_phase.set(Phase::Generation);
}

/// Pattern and Eliminate phases: remove full lines and collapse the blocks above them.
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
        assert_eq!(messages, [ScoreEvent::Single]);
    }

    fn is_game_over(world: &World) -> bool {
        matches!(
            world.resource::<NextState<Phase>>(),
            NextState::Pending(Phase::GameOver)
        )
    }

    fn lock_piece(tetrimino: Tetrimino, pos: Pos) -> World {
        let mut world = World::new();
        world.init_resource::<GameState>();
        world.init_resource::<NextState<Phase>>();
        world.insert_resource(GameAssets {
            impact: Handle::default(),
        });
        let root = world.spawn_empty().id();
        world.resource_mut::<GameState>().matrix.root_entity = root;
        world.spawn((CurrentPiece, tetrimino, Positioned(pos)));

        let result: Result = world.run_system_once(lock_down).unwrap();
        result.unwrap();
        world
    }

    #[test]
    fn leaving_gameplay_cleans_up_the_matrix() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin)
            .insert_state(Screen::Gameplay)
            .init_resource::<GameState>()
            .add_systems(OnExit(Screen::Gameplay), game_cleanup);
        app.world_mut().commands().queue(SpawnMatrix);
        app.update();

        let world = app.world_mut();
        let root = world.resource::<GameState>().matrix.root_entity;
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

    #[test]
    fn locking_adds_blocks_to_the_matrix() {
        let tetrimino = Tetrimino::from(TetriminoKind::T);
        let pos = Pos::new(4, 0);
        let world = lock_piece(tetrimino, pos);

        let matrix = &world.resource::<GameState>().matrix;
        for block_pos in tetrimino.block_positions(&pos) {
            let block = matrix.at_pos(block_pos);
            assert_eq!(world.get::<Positioned>(block).unwrap().0, block_pos);
        }
        assert_eq!(matrix.iter_non_empty().count(), 4);
    }

    #[test]
    fn locking_entirely_above_skyline_is_lock_out() {
        let world = lock_piece(TetriminoKind::I.into(), Pos::new(4, SKYLINE));
        assert!(is_game_over(&world));
    }

    #[test]
    fn locking_partially_above_skyline_is_not_game_over() {
        // The O piece covers this row and the one above
        let world = lock_piece(TetriminoKind::O.into(), Pos::new(4, SKYLINE - 1));
        assert!(!is_game_over(&world));
    }

    #[test]
    fn blocked_spawn_position_is_block_out() {
        let mut world = World::new();
        world.init_resource::<GameState>();
        world.init_resource::<NextState<Phase>>();
        world.spawn(NextTetriminoZone);
        for x in 0..MATRIX_WIDTH as i8 {
            spawn_block(&mut world, Pos::new(x, INITIAL_POS.y));
        }

        let result: Result = world.run_system_once(generate_piece).unwrap();
        result.unwrap();

        assert!(is_game_over(&world));
        assert!(
            world
                .query_filtered::<(), With<CurrentPiece>>()
                .iter(&world)
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

    #[test]
    fn new_piece_falls_even_if_previous_one_landed() {
        let mut world = World::new();
        let mut timers = Timers::default();
        // The previous piece landed, which pauses the fall timer
        timers.fall.pause();
        world.insert_resource(timers);
        world.spawn((CurrentPiece, Positioned(INITIAL_POS)));

        world.run_system_once(start_fall_timer).unwrap();

        assert!(!world.resource::<Timers>().fall.is_paused());
    }
}
