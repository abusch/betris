use std::time::Duration;

use bevy::{
    color::palettes::{self},
    ecs::component::StorageType,
    prelude::*,
    sprite::Anchor,
};
use input::Action;
use leafwing_input_manager::action_state::ActionState;
use spawners::{
    next_piece_zone::NextPieceDisplay, piece::CurrentPiece, SpawnMatrix, SpawnNextPieceZone,
    SpawnPiece,
};

use self::matrix::Matrix;
use crate::{
    pieces::{Bag, Piece},
    pos::Pos,
    screen::Screen,
};

mod input;
mod matrix;
pub mod spawners;

pub const MATRIX_WIDTH: usize = 10;
pub const MATRIX_HEIGHT: usize = 40;
pub const SCALE: f32 = 20.0;

pub fn plugin(app: &mut App) {
    app.init_state::<Phase>()
        .init_resource::<GameState>()
        .insert_resource(ClearColor(palettes::css::LIGHT_GRAY.into()))
        .add_plugins((input::plugin, spawners::plugin))
        .add_systems(OnEnter(Screen::Playing), game_setup)
        .add_systems(
            Update,
            (ui_update, debug_stuff).run_if(in_state(Screen::Playing)),
        )
        .add_systems(
            OnEnter(Phase::Generation),
            (clean_up_pieces, generate_piece).chain(),
        )
        .add_systems(
            Update,
            (debug_stuff, handle_input, update_piece_transform).run_if(in_state(Phase::Falling)),
        )
        .add_systems(OnEnter(Phase::Lock), handle_lock)
        .add_systems(OnEnter(Phase::Pattern), detect_patterns)
        .add_systems(OnEnter(Phase::Eliminate), eliminate);
}

// TODO Is this necessary? Should it be a bunch of serial systems?
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, States, strum::Display)]
pub enum Phase {
    Generation,
    Falling,
    Lock,
    Pattern,
    Eliminate,
    Completion,
    #[default]
    Noop,
}

#[derive(Default, Resource)]
pub struct GameState {
    pub matrix: Matrix,
    pub bag: Bag,
}

/// A mino (i.e block) which is part of a piece
#[derive(Component)]
pub struct Mino;

/// A static block that has been committed to the matrix.
pub struct Block;

impl Component for Block {
    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, entity, _component_id| {
            let pos = world
                .entity(entity)
                .get::<Pos>()
                .copied()
                .expect("Block component without Pos!");
            info!("Block was added at {pos}");
            let mut state = world.resource_mut::<GameState>();
            state.matrix.insert(pos, entity);
        });
    }

    const STORAGE_TYPE: StorageType = StorageType::Table;
}

#[derive(Bundle)]
pub struct MinoBundle {
    sprite: SpriteBundle,
    mino: Mino,
}

impl MinoBundle {
    pub fn new(pos: Pos, color: Color) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(1.0)),
                    anchor: Anchor::Center,
                    color,
                    ..default()
                },
                transform: Transform::from_xyz(pos.x as f32, pos.y as f32, 1.0),
                ..default()
            },
            mino: Mino,
        }
    }
}

#[derive(Bundle)]
pub struct BlockBundle {
    sprite: SpriteBundle,
    block: Block,
    pos: Pos,
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
            pos,
            block: Block,
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
struct FallTimer(Timer);

impl FallTimer {
    pub fn normal_drop(&mut self) {
        // TODO make it depend on the current game level
        self.0.set_duration(Duration::from_millis(1000));
        self.0.reset();
    }

    pub fn soft_drop(&mut self) {
        self.0.set_duration(Duration::from_millis(50));
        self.0.reset();
    }
}

// Marker component to update the phase debug
#[derive(Component)]
pub struct PhaseDebug;

fn game_setup(
    mut commands: Commands,
    mut next_phase: ResMut<NextState<Phase>>,
    asset_server: Res<AssetServer>,
) {
    let font_handle = asset_server.load("fonts/JetBrainsMonoNLNerdFont-Regular.ttf");
    commands.insert_resource(FallTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));

    commands.trigger(SpawnMatrix);
    commands.trigger(SpawnNextPieceZone);

    // Debug display
    let style = TextStyle {
        font: font_handle,
        font_size: 18.0,
        color: palettes::css::RED.into(),
    };
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new("Phase: ", style.clone()),
            TextSection::new(format!("{:?}", Phase::Generation), style),
        ]),
        PhaseDebug,
    ));
    next_phase.set(Phase::Generation);
}

fn ui_update(phase: Res<State<Phase>>, mut text: Query<&mut Text, With<PhaseDebug>>) {
    if let Ok(mut text) = text.get_single_mut() {
        text.sections[1].value = format!("{}", phase.get());
    }
}

fn clean_up_pieces(mut commands: Commands, pieces: Query<Entity, With<Piece>>) {
    for piece in pieces.into_iter() {
        info!("Despawning piece");
        commands.entity(piece).despawn_recursive();
    }
}

fn generate_piece(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    mut next_phase: ResMut<NextState<Phase>>,
    next_piece_zone: Query<Entity, With<NextPieceDisplay>>,
) {
    let next_piece_zone_entity = next_piece_zone.single();
    let piece: Piece = state.bag.pop_next().into();
    let next_piece: Piece = state.bag.peek_next().into();

    info!("Generating new piece {:?}", piece.typ);

    commands.trigger_targets(SpawnPiece::current(piece), state.matrix.root_entity);
    commands.trigger_targets(SpawnPiece::next(next_piece), next_piece_zone_entity);

    // TODO ghost piece
    next_phase.set(Phase::Falling);
}

fn handle_input(
    mut current_piece_query: Query<(&mut Piece, &mut Pos), With<CurrentPiece>>,
    state: Res<GameState>,
    action_state: Res<ActionState<Action>>,
    time: Res<Time>,
    mut fall_timer: ResMut<FallTimer>,
    mut next_phase: ResMut<NextState<Phase>>,
) {
    let (mut current_piece, mut pos) = current_piece_query.single_mut();

    if action_state.just_pressed(&Action::Down) {
        fall_timer.soft_drop();
    } else if action_state.just_released(&Action::Down) {
        fall_timer.normal_drop();
    }
    for _ in 0..fall_timer.tick(time.delta()).times_finished_this_tick() {
        let down_pos = pos.down();
        if current_piece.min_y(down_pos) >= 0 && state.matrix.is_pos_valid(&current_piece, down_pos)
        {
            *pos = down_pos;
        } else {
            next_phase.set(Phase::Lock);
            fall_timer.normal_drop();
            return;
        }
    }

    if action_state.just_pressed(&Action::RotateLeft) {
        current_piece.rotate_ccw();
    } else if action_state.just_pressed(&Action::RotateRight) {
        current_piece.rotate_cw();
    } else if action_state.just_pressed(&Action::Left) {
        let left_pos = pos.left();
        if current_piece.min_x(left_pos) >= 0 && state.matrix.is_pos_valid(&current_piece, left_pos)
        {
            *pos = left_pos;
        }
    } else if action_state.just_pressed(&Action::Right) {
        let right_pos = pos.right();
        if current_piece.max_x(right_pos) <= 9
            && state.matrix.is_pos_valid(&current_piece, right_pos)
        {
            *pos = right_pos;
        }
    } else if action_state.just_pressed(&Action::Drop) {
        next_phase.set(Phase::Generation);
    }
}

/// Update the piece's Transform based on its grid position.
///
/// The piece's position (as tracked by `Pos`) is the position (in grid coordinates) of the "visual
/// center" of the piece. The blocks that make up the piece will be positioned relative to that.
fn update_piece_transform(
    mut piece: Query<(&mut Transform, Ref<Pos>, Ref<Piece>), With<CurrentPiece>>,
) {
    if let Ok((mut transform, pos, piece)) = piece.get_single_mut() {
        // If the position of the piece has changed, update its transform
        if pos.is_changed() {
            transform.translation.x = pos.x as f32;
            transform.translation.y = pos.y as f32;
        }
        if piece.is_changed() {
            transform.rotation = Quat::from_rotation_z(piece.orientation.angle());
        }
    }
}

fn handle_lock(
    mut commands: Commands,
    state: Res<GameState>,
    current_piece: Query<(&Pos, &Piece), With<CurrentPiece>>,
    mut next_phase: ResMut<NextState<Phase>>,
) {
    if let Ok((piece_pos, piece)) = current_piece.get_single() {
        info!("Locking piece");
        commands
            .entity(state.matrix.root_entity)
            .with_children(|children| {
                for block_pos in piece.block_positions(*piece_pos) {
                    children.spawn(BlockBundle::new(block_pos));
                }
            });
    }

    next_phase.set(Phase::Pattern);
}

fn detect_patterns(
    mut commands: Commands,
    state: ResMut<GameState>,
    mut next_phase: ResMut<NextState<Phase>>,
) {
    let lines = state.matrix.full_lines();
    if !lines.is_empty() {
        info!("Lines completed! {lines:?}");
    }
    next_phase.set(Phase::Eliminate);
}

fn eliminate(
    mut commands: Commands,
    state: Res<GameState>,
    mut next_phase: ResMut<NextState<Phase>>,
) {
    next_phase.set(Phase::Generation);
}

fn debug_stuff(mut gizmos: Gizmos) {
    gizmos
        .grid_2d(
            Vec2::new(-5.5 * SCALE, 0.5 * SCALE),
            0.0,
            UVec2::new(10, 22),
            Vec2::new(SCALE, SCALE),
            palettes::css::HOT_PINK,
        )
        .outer_edges();
}
