use std::time::Duration;

use bevy::{
    color::palettes::css::{BLACK, GRAY, WHITE},
    ecs::component::{Mutable, StorageType},
    prelude::*,
};
use bevy_enhanced_input::{events::ActionEvents, prelude::Actions};
use bevy_tween::{
    bevy_time_runner::TimeRunnerEnded,
    prelude::{AnimationBuilderExt, EaseKind},
    tween::TargetComponent,
};
use bevy_vector_shapes::shapes::ShapeFill;
use input::{HardDrop, InGame, Left, Right, RotateLeft, RotateRight, SoftDrop};
use score::ScoreEvent;
use spawners::{
    INITIAL_POS, Positioned, SpawnMatrix, SpawnNextZone, SpawnPiece,
    next_zone::NextTetriminoZone,
    piece::{CurrentPiece, GhostPiece, Mino, SpawnMino},
};
use timers::Timers;

use self::matrix::Matrix;
use crate::{
    model::{Bag, Tetrimino},
    screen::Screen,
    tweening::shape_color_to,
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
    app.init_state::<Phase>()
        .init_resource::<GameState>()
        .register_type::<GameState>()
        .insert_resource(ClearColor(BLACK.into()))
        .add_systems(OnEnter(Screen::Gameplay), game_setup)
        .add_systems(
            OnEnter(Phase::Generation),
            (clean_up_pieces, generate_piece).chain(),
        )
        .add_systems(OnExit(Phase::Generation), first_drop)
        .add_systems(OnEnter(Phase::Falling), start_fall_timer)
        .add_systems(
            Update,
            (
                tick_timers,
                handle_input,
                update_ghost,
                update_piece_transform,
            )
                .chain()
                .run_if(in_state(Phase::Falling)),
        )
        .add_systems(OnEnter(Phase::Lock), handle_lock)
        .add_systems(OnEnter(Phase::Pattern), detect_patterns)
        .add_systems(OnEnter(Phase::Animate), animate)
        .add_systems(Update, animate_done.run_if(in_state(Phase::Animate)))
        .add_systems(OnEnter(Phase::Eliminate), eliminate)
        .add_systems(OnExit(Phase::Eliminate), update_blocks_transform)
        .add_systems(OnExit(Screen::Gameplay), game_cleanup);

    app.add_plugins((input::plugin, spawners::plugin, score::plugin, ui::plugin));

    #[cfg(feature = "dev")]
    app.add_plugins(debug::plugin);
    // app.add_plugins(ResourceInspectorPlugin::<GameState>::default());
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, States, strum::Display)]
pub enum Phase {
    Generation,
    Falling,
    Lock,
    Pattern,
    Animate,
    Eliminate,
    Completion,
    #[default]
    Noop,
}

#[derive(Default, Resource, Reflect)]
pub struct GameState {
    pub matrix: Matrix,
    pub bag: Bag,
}

/// A static block that has been committed to the matrix.
pub struct Block;

impl Component for Block {
    type Mutability = Mutable;
    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, ctx| {
            let pos = world
                .entity(ctx.entity)
                .get::<Positioned>()
                .copied()
                .expect("Block component without Pos!");
            info!("Block was added at {}", *pos);
            let mut state = world.resource_mut::<GameState>();
            state.matrix.insert(*pos, ctx.entity);
        });
    }

    const STORAGE_TYPE: StorageType = StorageType::Table;
}

fn game_setup(
    mut commands: Commands,
    mut next_phase: ResMut<NextState<Phase>>,
    mut event_writer: EventWriter<ScoreEvent>,
) {
    commands.init_resource::<Timers>();

    commands.queue(SpawnMatrix);
    commands.queue(SpawnNextZone);

    event_writer.write(ScoreEvent::LevelStart(1));

    next_phase.set(Phase::Generation);
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

    commands.queue(SpawnPiece::current(tetrimino).with_parent(state.matrix.root_entity));
    let ghost_pos = state.matrix.lowest_valid_pos(&tetrimino, &INITIAL_POS);
    commands.queue(SpawnPiece::ghost(tetrimino, ghost_pos).with_parent(state.matrix.root_entity));

    commands.queue(SpawnPiece::next(next_piece).with_parent(next_zone_entity));

    next_phase.set(Phase::Falling);

    Ok(())
}

fn start_fall_timer(mut timers: ResMut<Timers>) {
    info!("Starting fall timer");
    timers.fall.normal_drop();
}

fn first_drop(
    mut current_piece_query: Query<(&mut Tetrimino, &mut Positioned), With<CurrentPiece>>,
    state: Res<GameState>,
) -> Result {
    let (current_piece, mut pos) = current_piece_query.single_mut()?;
    let down_pos = pos.down();
    if current_piece.min_y(&down_pos) >= 0 && state.matrix.is_pos_valid(&current_piece, &down_pos) {
        **pos = down_pos;
    }

    Ok(())
}

fn tick_timers(mut timers: ResMut<Timers>, time: Res<Time>) {
    timers.tick(time.delta());
}

fn handle_input(
    mut current_piece_query: Query<(&mut Tetrimino, &mut Positioned), With<CurrentPiece>>,
    state: Res<GameState>,
    action_state: Single<&Actions<InGame>>,
    mut timers: ResMut<Timers>,
    mut next_phase: ResMut<NextState<Phase>>,
) -> Result {
    let action_state = action_state.into_inner();
    let (mut current_piece, mut pos) = current_piece_query.single_mut()?;

    // If lock timer has expired -> move to LOCK state
    if timers.lock.times_finished_this_tick() > 0 {
        next_phase.set(Phase::Lock);
        return Ok(());
    }

    if timers.lock.paused() {
        for _ in 0..timers.fall.times_finished_this_tick() {
            let down_pos = pos.down();
            if state.matrix.is_pos_valid(&current_piece, &down_pos) {
                **pos = down_pos;
            }
        }
    }

    if action_state.action::<RotateLeft>().events() == ActionEvents::STARTED | ActionEvents::FIRED {
        let rotated = current_piece.rotated_ccw();
        if state.matrix.is_pos_valid(&rotated, &pos) {
            *current_piece = rotated;
        }
    }
    if action_state.action::<RotateRight>().events() == ActionEvents::STARTED | ActionEvents::FIRED
    {
        let rotated = current_piece.rotated_cw();
        if state.matrix.is_pos_valid(&rotated, &pos) {
            *current_piece = rotated;
        }
    }

    if action_state.action::<Left>().events() == ActionEvents::STARTED | ActionEvents::FIRED {
        let left_pos = pos.left();
        if current_piece.min_x(&left_pos) >= 0
            && state.matrix.is_pos_valid(&current_piece, &left_pos)
        {
            **pos = left_pos;
        }
    } else if action_state.action::<Right>().events() == ActionEvents::STARTED | ActionEvents::FIRED
    {
        let right_pos = pos.right();
        if current_piece.max_x(&right_pos) <= 9
            && state.matrix.is_pos_valid(&current_piece, &right_pos)
        {
            **pos = right_pos;
        }
    }
    if action_state.action::<HardDrop>().events() == ActionEvents::STARTED | ActionEvents::FIRED {
        **pos = state.matrix.lowest_valid_pos(&current_piece, &pos);
        next_phase.set(Phase::Lock);
        return Ok(());
    }
    if action_state.action::<SoftDrop>().events() == ActionEvents::STARTED | ActionEvents::FIRED {
        timers.fall.soft_drop();
    } else if action_state.action::<SoftDrop>().events() == ActionEvents::COMPLETED {
        timers.fall.normal_drop();
    }

    if state.matrix.is_on_surface(&current_piece, &pos) {
        // If we just landed on a surface, kick off the lock timer
        if timers.lock.paused() {
            info!("Starting lock timer!");
            timers.fall.pause();
            timers.lock.reset();
            timers.lock.unpause();
        }
    } else {
        // If we were in lock phase but are free to fall, go back to "falling" phase
        if !timers.lock.paused() {
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

/// Update the piece's Transform based on its grid position.
///
/// The piece's position (as tracked by `Pos`) is the position (in grid coordinates) of the "visual
/// center" of the piece. The blocks that make up the piece will be positioned relative to that.
fn update_piece_transform(
    mut pieces: Query<(&mut Transform, Ref<Positioned>, Ref<Tetrimino>, &Children), Without<Mino>>,
    mut blocks: Query<&mut Transform, With<Mino>>,
) {
    for (mut transform, pos, piece, children) in pieces.iter_mut() {
        // If the position of the piece has changed, update its transform
        if pos.is_changed() {
            info!("Updating current piece transform");
            *transform = pos.0.into();
        }
        if piece.is_changed() {
            info!("Updating current piece's blocks transform");
            for (child, offset) in children.iter().zip(piece.block_offsets()) {
                if let Ok(mut transform) = blocks.get_mut(child) {
                    *transform = offset.into();
                }
            }
        }
    }
}

fn handle_lock(
    mut commands: Commands,
    state: Res<GameState>,
    current_piece: Query<(&Positioned, &Tetrimino), With<CurrentPiece>>,
    mut next_phase: ResMut<NextState<Phase>>,
) {
    if let Ok((piece_pos, piece)) = current_piece.single() {
        info!("Locking piece");

        commands
            .entity(state.matrix.root_entity)
            .with_children(|children| {
                for block_pos in piece.block_positions(piece_pos) {
                    // children.spawn(BlockBundle::new(block_pos));
                    children
                        .spawn((Block, Positioned(block_pos)))
                        .queue(SpawnMino(block_pos, Some(GRAY.into())));
                }
            });
    }

    next_phase.set(Phase::Pattern);
}

#[derive(Component)]
pub struct ToDelete;

fn detect_patterns(
    mut commands: Commands,
    state: ResMut<GameState>,
    mut next_phase: ResMut<NextState<Phase>>,
) {
    let mut has_deletions = false;
    for e in state.matrix.entities_to_delete() {
        info!("Marking block {e} for deletion");
        commands.entity(e).insert(ToDelete);
        has_deletions = true;
    }

    if has_deletions {
        next_phase.set(Phase::Animate);
    } else {
        // if there is nothing to delete, go straight back to the Generation phase
        next_phase.set(Phase::Generation);
    }
}

#[derive(Component)]
pub struct Animator;

fn animate(
    mut commands: Commands,
    to_delete: Query<(Entity, &Children), With<ToDelete>>,
    to_animate: Query<Entity, With<ShapeFill>>,
) {
    info!("Start animation");
    let mut target = Vec::new();
    for (_, children) in to_delete.iter() {
        for child in children {
            if let Ok(z) = to_animate.get(*child) {
                target.push(z);
            }
        }
    }
    if target.is_empty() {
        warn!("No ShapeFill found!");
    }
    let entities = TargetComponent::from(target);

    commands.spawn(Animator).animation().insert_tween_here(
        Duration::from_secs_f32(0.3),
        EaseKind::QuadraticOut,
        entities
            .state(WHITE.with_alpha(1.0).into())
            .with(shape_color_to(Color::srgb(5.0, 5.0, 5.0))),
    );
}

fn animate_done(mut next_phase: ResMut<NextState<Phase>>, mut ended: EventReader<TimeRunnerEnded>) {
    for ended in ended.read() {
        if ended.is_completed() {
            info!("Animation completed! Moving to Eliminate phase");
            next_phase.set(Phase::Eliminate);
        }
    }
}

fn eliminate(
    mut commands: Commands,
    to_delete: Query<Entity, With<ToDelete>>,
    mut state: ResMut<GameState>,
    mut next_phase: ResMut<NextState<Phase>>,
    mut event_writer: EventWriter<ScoreEvent>,
) {
    // Despawn entities that were deleted
    for e in to_delete.iter() {
        info!("Despawning block {e}");
        commands.entity(e).despawn();
    }

    // Remove lines from the matrix
    let mut lines = state.matrix.full_lines();
    let num_lines = lines.len();
    lines.reverse();
    for line in lines {
        info!("Removing line {line}");
        state.matrix.delete_line(line);
    }

    // Reflect new positions
    for (pos, entity) in state.matrix.iter_non_empty() {
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert(Positioned(pos));
        } else {
            warn!("Missing entity {entity}");
        }
    }

    match num_lines {
        0 => (),
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

    next_phase.set(Phase::Generation);
}

fn update_blocks_transform(mut blocks: Query<(&mut Transform, &Positioned), With<Block>>) {
    for (mut transform, pos) in blocks.iter_mut() {
        // If the position of the block has changed, update its transform
        *transform = pos.0.into();
    }
}
