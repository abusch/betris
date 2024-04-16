use std::time::Duration;

use bevy::{prelude::*, sprite::Anchor};

use crate::{
    pieces::{Bag, Piece},
    AppState, Pos,
};

pub const SCALE: f32 = 20.0;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<Phase>()
            .add_systems(OnEnter(AppState::InGame), game_setup)
            .add_systems(Update, ui_update.run_if(in_state(AppState::InGame)))
            .add_systems(
                OnEnter(Phase::Generation),
                (clean_up_pieces, generate_piece).chain(),
            )
            .add_systems(
                Update,
                (
                    handle_input,
                    update_piece_transform,
                    update_blocks_transforms,
                )
                    .run_if(in_state(Phase::Falling)),
            )
            .add_systems(OnEnter(Phase::Lock), handle_lock);
    }
}

// TODO Is this necessary? Should it be a bunch of serial systems?
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

/// Parent component of the Matrix i.e the main grid where the game happens
#[derive(Component)]
pub struct Matrix;

/// The parent component of where the next piece is displayed
#[derive(Component)]
pub struct NextPieceDisplay;

/// Marker component for the current piece (i.e. the piece controlled by the player)
#[derive(Component)]
pub struct CurrentPiece;

/// Marker component for the next piece (i.e. the piece displayed in the "next piece" zone)
#[derive(Component)]
pub struct NextPiece;

#[derive(Component)]
pub struct Block;

#[derive(Bundle)]
pub struct MinoBundle {
    sprite: SpriteBundle,
    block: Block,
}

impl MinoBundle {
    pub fn new(pos: Pos, color: Color) -> Self {
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(1.0)),
                    anchor: Anchor::TopLeft,
                    color,
                    ..default()
                },
                transform: Transform::from_xyz(pos.x as f32, pos.y as f32, 1.0),
                ..default()
            },
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
    commands.init_resource::<Bag>();
    commands.insert_resource(FallTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));

    // Matrix i.e main game area
    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(-200.0, -200.0, 1.0)
                    .with_scale(Vec3::new(SCALE, SCALE, 1.0)),
                ..default()
            },
            Matrix,
        ))
        .with_children(|children| {
            // "floor"
            children.spawn(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(10.0, 1.0)),
                    anchor: Anchor::TopLeft,
                    ..default()
                },
                // Floor is unit below "zero"
                transform: Transform::from_xyz(0.0, -1.0, 1.0),
                ..default()
            });
        })
        .with_children(|parent| {
            parent.spawn(MinoBundle::new(Pos::new(5, 5), Color::PINK));
        });

    // Next-piece display zone
    commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(100.0, 100.0, 1.0).with_scale(Vec3::new(
                SCALE / 2.0,
                SCALE / 2.0,
                1.0,
            )),
            ..default()
        },
        NextPieceDisplay,
    ));

    // Debug display
    let style = TextStyle {
        font: font_handle,
        font_size: 18.0,
        color: Color::RED,
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

fn clean_up_pieces(
    mut commands: Commands,
    current_piece: Query<Entity, With<CurrentPiece>>,
    next_piece: Query<Entity, With<NextPiece>>,
) {
    if let Ok(current_piece) = current_piece.get_single() {
        info!("Cleaning up current piece");
        commands.entity(current_piece).despawn_recursive();
    }
    if let Ok(next_piece) = next_piece.get_single() {
        info!("Cleaning up next piece");
        commands.entity(next_piece).despawn_recursive();
    }
}

fn generate_piece(
    mut commands: Commands,
    mut bag: ResMut<Bag>,
    mut next_phase: ResMut<NextState<Phase>>,
    matrix: Query<Entity, With<Matrix>>,
    next_piece_zone: Query<Entity, With<NextPieceDisplay>>,
) {
    let matrix_entity = matrix.single();
    let next_piece_zone_entity = next_piece_zone.single();
    let piece: Piece = bag.pop_next().into();
    let next_piece: Piece = bag.peek_next().into();
    let initial_pos = Pos::new(2, 10);

    info!("Generating new piece {:?}", piece.typ);

    commands
        .entity(next_piece_zone_entity)
        .despawn_descendants();

    spawn_piece(
        &mut commands,
        matrix_entity,
        piece,
        initial_pos,
        CurrentPiece,
    );
    spawn_piece(
        &mut commands,
        next_piece_zone_entity,
        next_piece,
        Pos::new(0, 0),
        NextPiece,
    );

    // TODO setup next piece + ghose piece
    next_phase.set(Phase::Falling);
}

fn spawn_piece<T: Component>(
    commands: &mut Commands,
    parent: Entity,
    piece: Piece,
    initial_pos: Pos,
    component: T,
) {
    commands.entity(parent).with_children(|matrix_children| {
        matrix_children
            .spawn((SpatialBundle::default(), initial_pos, piece, component))
            .with_children(|children| {
                for pos in piece.block_positions(initial_pos) {
                    children.spawn(MinoBundle::new(pos, piece.color()));
                }
            });
    });
}

fn handle_input(
    key: Res<Input<KeyCode>>,
    mut current_piece_query: Query<(&mut Piece, &mut Pos), With<CurrentPiece>>,
    time: Res<Time>,
    mut fall_timer: ResMut<FallTimer>,
    mut next_phase: ResMut<NextState<Phase>>,
) {
    let (mut current_piece, mut pos) = current_piece_query.single_mut();

    if key.just_pressed(KeyCode::Down) {
        fall_timer.soft_drop();
    } else if key.just_released(KeyCode::Down) {
        fall_timer.normal_drop();
    }
    for _ in 0..fall_timer.tick(time.delta()).times_finished_this_tick() {
        let down_pos = pos.down();
        if current_piece.min_y(down_pos) >= 0 {
            *pos = down_pos;
        } else {
            next_phase.set(Phase::Lock);
        }
    }

    if key.just_pressed(KeyCode::Z) {
        current_piece.rotate_ccw();
    } else if key.just_pressed(KeyCode::X) {
        current_piece.rotate_cw();
    } else if key.just_pressed(KeyCode::Left) {
        let left_pos = pos.left();
        if current_piece.min_x(left_pos) >= 0 {
            *pos = left_pos;
        }
    } else if key.just_pressed(KeyCode::Right) {
        let right_pos = pos.right();
        if current_piece.max_x(right_pos) <= 9 {
            *pos = right_pos;
        }
    } else if key.just_pressed(KeyCode::Space) {
        next_phase.set(Phase::Generation);
    }
}

/// Update the piece's Transform based on its grid position.
///
/// The piece's position (as tracked by `Pos`) is the position (in grid coordinates) of the "visual
/// center" of the piece. The blocks that make up the piece will be positioned relative to that.
fn update_piece_transform(mut piece: Query<(&mut Transform, Ref<Pos>), With<CurrentPiece>>) {
    if let Ok((mut transform, pos)) = piece.get_single_mut() {
        // If the position of the piece has changed, update its transform
        if pos.is_changed() {
            info!("Updating transform for piece");
            transform.translation.x = pos.x as f32;
            transform.translation.y = pos.y as f32;
        }
    }
}

/// Update the current piece's blocks Transforms based on the current facing.
///
/// A block's transform is relative to the piece itself, with the "visual rotation center" having
/// coordinates (0, 0). So a block with position (1, 0) will be to the right of the visual center.
fn update_blocks_transforms(
    mut piece: Query<(Ref<Piece>, &Children), With<CurrentPiece>>,
    mut blocks: Query<&mut Transform, With<Block>>,
) {
    if let Ok((piece, children)) = piece.get_single_mut() {
        // If the piece itself has changed (i.e. its orientation), reset the local transform of
        // its blocks
        if piece.is_changed() {
            info!("Updating blocks for piece = {:?}", piece.typ);
            for (child, block_pos) in children.iter().zip(piece.block_offsets().iter()) {
                if let Ok(mut transform) = blocks.get_mut(*child) {
                    transform.translation.x = block_pos.0 as f32;
                    transform.translation.y = block_pos.1 as f32;
                }
            }
        }
    }
}

fn handle_lock(
    mut commands: Commands,
    mut current_piece: Query<(&Pos, &Piece), With<CurrentPiece>>,
    board: Query<Entity, With<Matrix>>,
    mut next_phase: ResMut<NextState<Phase>>,
) {
    let matrix = board.single();
    if let Ok((piece_pos, piece)) = current_piece.get_single_mut() {
        // info!("Locking piece! Piece pos={piece_pos:?}");
        // commands.entity(matrix).with_children(|parent| {
        //     for block_pos in piece.block_positions(*piece_pos) {
        //         parent.spawn(bundle)
        //     }
        // });
        // if let Ok((mut transform, mut block_pos)) = blocks.get_mut(*child) {
        //     block_pos.x += piece_pos.x;
        //     block_pos.y += piece_pos.y;
        //     info!("Block pos = {block_pos:?}");
        //     commands.entity(*child).remove_parent_in_place();
        //     commands.entity(matrix).add_child(*child);
        //     transform.translation.x = block_pos.x as f32;
        //     transform.translation.y = block_pos.y as f32;
        // }
    }

    next_phase.set(Phase::Generation);
}
