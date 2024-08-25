use bevy::{
    color::palettes::{self},
    ecs::component::StorageType,
    prelude::*,
    sprite::Anchor,
};
use input::Action;
use leafwing_input_manager::action_state::ActionState;
use score::ScoreEvent;
use spawners::{
    next_zone::NextTetriminoZone,
    piece::{CurrentPiece, Mino},
    Positioned, SpawnMatrix, SpawnNextZone, SpawnPiece,
};
use timers::{FallTimer, LockTimer};

use self::matrix::Matrix;
use crate::{
    model::Pos,
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
    app.init_state::<Phase>()
        .init_resource::<GameState>()
        .register_type::<GameState>()
        .insert_resource(ClearColor(palettes::css::LIGHT_GRAY.into()))
        .add_systems(OnEnter(Screen::Playing), game_setup)
        .add_systems(
            OnEnter(Phase::Generation),
            (clean_up_pieces, generate_piece).chain(),
        )
        .add_systems(OnExit(Phase::Generation), first_drop)
        .add_systems(OnEnter(Phase::Falling), start_fall_timer)
        .add_systems(
            Update,
            (tick_timers, handle_input, update_piece_transform)
                .chain()
                .run_if(in_state(Phase::Falling)),
        )
        .add_systems(OnEnter(Phase::Lock), handle_lock)
        .add_systems(OnEnter(Phase::Pattern), detect_patterns)
        .add_systems(OnEnter(Phase::Eliminate), eliminate)
        .add_systems(OnExit(Phase::Eliminate), update_blocks_transform)
        .add_systems(OnExit(Screen::Playing), game_cleanup);

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
    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, entity, _component_id| {
            let pos = world
                .entity(entity)
                .get::<Positioned>()
                .copied()
                .expect("Block component without Pos!");
            info!("Block was added at {}", *pos);
            let mut state = world.resource_mut::<GameState>();
            state.matrix.insert(*pos, entity);
        });
    }

    const STORAGE_TYPE: StorageType = StorageType::Table;
}

#[derive(Bundle)]
pub struct BlockBundle {
    sprite: SpriteBundle,
    block: Block,
    pos: Positioned,
}

impl BlockBundle {
    pub fn new(pos: Pos) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(1.0)),
                    anchor: Anchor::Center,
                    color: palettes::css::GRAY.into(),
                    ..default()
                },
                transform: Transform::from_xyz(pos.x as f32, pos.y as f32, 1.0),
                ..default()
            },
            pos: Positioned(pos),
            block: Block,
        }
    }
}

fn game_setup(
    mut commands: Commands,
    mut next_phase: ResMut<NextState<Phase>>,
    mut event_writer: EventWriter<ScoreEvent>,
) {
    commands.init_resource::<FallTimer>();
    commands.init_resource::<LockTimer>();

    commands.trigger(SpawnMatrix);
    commands.trigger(SpawnNextZone);

    event_writer.send(ScoreEvent::LevelStart(1));

    next_phase.set(Phase::Generation);
}

fn game_cleanup(mut commands: Commands) {
    commands.remove_resource::<FallTimer>();
    commands.remove_resource::<LockTimer>();
}

fn clean_up_pieces(mut commands: Commands, pieces: Query<Entity, With<Tetrimino>>) {
    for piece in pieces.into_iter() {
        info!("Despawning tetrimino");
        commands.entity(piece).despawn_recursive();
    }
}

fn generate_piece(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut next_phase: ResMut<NextState<Phase>>,
    next_zone: Query<Entity, With<NextTetriminoZone>>,
) {
    let next_zone_entity = next_zone.single();
    let tetrimino: Tetrimino = state.bag.pop_next().into();
    let next_piece: Tetrimino = state.bag.peek_next().into();

    info!("Generating new tetrimino {:?}", tetrimino.kind);

    commands.trigger_targets(SpawnPiece::current(tetrimino), state.matrix.root_entity);
    commands.trigger_targets(SpawnPiece::next(next_piece), next_zone_entity);

    // TODO ghost piece
    next_phase.set(Phase::Falling);
}

fn start_fall_timer(mut fall_timer: ResMut<FallTimer>) {
    info!("Starting fall timer");
    fall_timer.normal_drop();
}

fn first_drop(
    mut current_piece_query: Query<(&mut Tetrimino, &mut Positioned), With<CurrentPiece>>,
    state: Res<GameState>,
) {
    let (current_piece, mut pos) = current_piece_query.single_mut();
    let down_pos = pos.down();
    if current_piece.min_y(&down_pos) >= 0 && state.matrix.is_pos_valid(&current_piece, &down_pos) {
        **pos = down_pos;
    }
}

fn tick_timers(
    mut fall_timer: ResMut<FallTimer>,
    mut lock_timer: ResMut<LockTimer>,
    time: Res<Time>,
) {
    fall_timer.tick(time.delta());
    lock_timer.tick(time.delta());
}

fn handle_input(
    mut current_piece_query: Query<(&mut Tetrimino, &mut Positioned), With<CurrentPiece>>,
    state: Res<GameState>,
    action_state: Res<ActionState<Action>>,
    mut fall_timer: ResMut<FallTimer>,
    mut lock_timer: ResMut<LockTimer>,
    mut next_phase: ResMut<NextState<Phase>>,
) {
    let (mut current_piece, mut pos) = current_piece_query.single_mut();

    // If lock timer has expired -> move to LOCK state
    if lock_timer.times_finished_this_tick() > 0 {
        next_phase.set(Phase::Lock);
        return;
    }

    if lock_timer.paused() {
        for _ in 0..fall_timer.times_finished_this_tick() {
            let down_pos = pos.down();
            if state.matrix.is_pos_valid(&current_piece, &down_pos) {
                **pos = down_pos;
            }
        }
    }

    if action_state.just_pressed(&Action::RotateLeft) {
        let rotated = current_piece.rotated_ccw();
        if state.matrix.is_pos_valid(&rotated, &pos) {
            *current_piece = rotated;
        }
    } else if action_state.just_pressed(&Action::RotateRight) {
        let rotated = current_piece.rotated_cw();
        if state.matrix.is_pos_valid(&rotated, &pos) {
            *current_piece = rotated;
        }
    }
    if action_state.just_pressed(&Action::Left) {
        let left_pos = pos.left();
        if current_piece.min_x(&left_pos) >= 0
            && state.matrix.is_pos_valid(&current_piece, &left_pos)
        {
            **pos = left_pos;
        }
    } else if action_state.just_pressed(&Action::Right) {
        let right_pos = pos.right();
        if current_piece.max_x(&right_pos) <= 9
            && state.matrix.is_pos_valid(&current_piece, &right_pos)
        {
            **pos = right_pos;
        }
    }
    if action_state.just_pressed(&Action::HardDrop) {
        **pos = state.matrix.lowest_valid_pos(&current_piece, &pos);
        next_phase.set(Phase::Lock);
        return;
    }
    if action_state.just_pressed(&Action::SoftDrop) {
        fall_timer.soft_drop();
    } else if action_state.just_released(&Action::SoftDrop) {
        fall_timer.normal_drop();
    }

    if state.matrix.is_on_surface(&current_piece, &pos) {
        // If we just landed on a surface, kick off the lock timer
        if lock_timer.paused() {
            info!("Starting lock timer!");
            fall_timer.pause();
            lock_timer.reset();
            lock_timer.unpause();
        }
    } else {
        // If we were in lock phase but are free to fall, go back to "falling" phase
        if !lock_timer.paused() {
            lock_timer.pause();
            fall_timer.normal_drop();
            fall_timer.unpause();
        }
    }
}

/// Update the piece's Transform based on its grid position.
///
/// The piece's position (as tracked by `Pos`) is the position (in grid coordinates) of the "visual
/// center" of the piece. The blocks that make up the piece will be positioned relative to that.
fn update_piece_transform(
    mut piece: Query<
        (&mut Transform, Ref<Positioned>, Ref<Tetrimino>, &Children),
        With<CurrentPiece>,
    >,
    mut blocks: Query<&mut Transform, (With<Mino>, Without<CurrentPiece>)>,
) {
    if let Ok((mut transform, pos, piece, children)) = piece.get_single_mut() {
        // If the position of the piece has changed, update its transform
        if pos.is_changed() {
            info!("Updating current piece transform");
            transform.translation.x = pos.x as f32;
            transform.translation.y = pos.y as f32;
        }
        if piece.is_changed() {
            info!("Updating current piece's blocks transform");
            for (child, offset) in children.iter().zip(piece.block_offsets()) {
                if let Ok(mut transform) = blocks.get_mut(*child) {
                    transform.translation.x = offset.x as f32;
                    transform.translation.y = offset.y as f32;
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
    if let Ok((piece_pos, piece)) = current_piece.get_single() {
        info!("Locking piece");
        commands
            .entity(state.matrix.root_entity)
            .with_children(|children| {
                for block_pos in piece.block_positions(piece_pos) {
                    children.spawn(BlockBundle::new(block_pos));
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
    for e in state.matrix.entities_to_delete() {
        info!("Marking block {e} for deletion");
        commands.entity(e).insert(ToDelete);
    }
    next_phase.set(Phase::Eliminate);
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
        commands.entity(e).despawn_recursive();
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
        if let Some(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.insert(Positioned(pos));
        } else {
            warn!("Missing entity {entity}");
        }
    }

    match num_lines {
        0 => (),
        1 => {
            event_writer.send(ScoreEvent::Single);
        }
        2 => {
            event_writer.send(ScoreEvent::Double);
        }
        3 => {
            event_writer.send(ScoreEvent::Triple);
        }
        4 => {
            event_writer.send(ScoreEvent::Tetris);
        }
        n => warn!("How did we complete {n} lines?!?"),
    }

    next_phase.set(Phase::Generation);
}

fn update_blocks_transform(mut blocks: Query<(&mut Transform, &Positioned), With<Block>>) {
    for (mut transform, pos) in blocks.iter_mut() {
        // If the position of the block has changed, update its transform
        info!("Updating transform for block at pos {}", **pos);
        transform.translation.x = pos.x as f32;
        transform.translation.y = pos.y as f32;
    }
}
