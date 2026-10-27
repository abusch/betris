use bevy::prelude::*;

use crate::model::{Facing, Pos, Rotation, Tetrimino, TetriminoKind};

use super::{MATRIX_HEIGHT, MATRIX_WIDTH, score::TSpin};

#[derive(Debug, Clone, Reflect)]
pub struct Matrix {
    pub root_entity: Entity,
    pub board: [Entity; MATRIX_WIDTH as usize * MATRIX_HEIGHT as usize],
}

impl Default for Matrix {
    fn default() -> Self {
        Self::new()
    }
}

impl Matrix {
    pub fn new() -> Self {
        Self {
            root_entity: Entity::PLACEHOLDER,
            board: [Entity::PLACEHOLDER; MATRIX_WIDTH as usize * MATRIX_HEIGHT as usize],
        }
    }

    pub fn is_pos_valid(&self, tetrimino: &Tetrimino, pos: &Pos) -> bool {
        for p in tetrimino.block_positions(pos) {
            if p.x < 0
                || p.x >= (MATRIX_WIDTH as i8)
                || p.y < 0
                || self.at_pos(p) != Entity::PLACEHOLDER
            {
                return false;
            }
        }
        true
    }

    pub fn is_on_surface(&self, tetrimino: &Tetrimino, pos: &Pos) -> bool {
        !self.is_pos_valid(tetrimino, &pos.down())
    }

    pub fn lowest_valid_pos(&self, tetrimino: &Tetrimino, pos: &Pos) -> Pos {
        let mut p = *pos;

        while !self.is_on_surface(tetrimino, &p) {
            p = p.down();
        }
        p
    }

    /// Rotate `piece` at `pos` with the Super Rotation System: the first of its five Rotation
    /// Points where the rotated piece fits is used. Returns the rotated piece, its new position and
    /// the Rotation Point used (1 to 5), or `None` if the piece can't rotate.
    pub fn rotate(
        &self,
        piece: &Tetrimino,
        pos: &Pos,
        rotation: Rotation,
    ) -> Option<(Tetrimino, Pos, u8)> {
        let rotated = piece.rotated(rotation);
        piece
            .rotation_points(rotation)
            .iter()
            .zip(1..)
            .map(|(&offset, point)| (*pos + offset, point))
            .find(|(new_pos, _)| self.is_pos_valid(&rotated, new_pos))
            .map(|(new_pos, point)| (rotated, new_pos, point))
    }

    /// Recognise a T-Spin by the T-Tetrimino locking down at `pos`, using the cells diagonally
    /// adjacent to its center (see section 9.1 of the guideline). The caller must check that the
    /// piece's last move was a rotation, using `rotation_point`.
    pub fn t_spin(&self, piece: &Tetrimino, pos: &Pos, rotation_point: u8) -> Option<TSpin> {
        if piece.kind != TetriminoKind::T {
            return None;
        }

        // Corners A and B are on the side the T points to, C and D on the flat side
        let (front, back) = match piece.facing {
            Facing::North => ([(-1, 1), (1, 1)], [(-1, -1), (1, -1)]),
            Facing::East => ([(1, 1), (1, -1)], [(-1, 1), (-1, -1)]),
            Facing::South => ([(-1, -1), (1, -1)], [(-1, 1), (1, 1)]),
            Facing::West => ([(-1, 1), (-1, -1)], [(1, 1), (1, -1)]),
        };
        let occupied = |corners: [(i8, i8); 2]| {
            corners
                .into_iter()
                .filter(|&corner| self.is_occupied(*pos + corner.into()))
                .count()
        };

        match (occupied(front), occupied(back)) {
            (2, 1..) => Some(TSpin::Full),
            // Kicking into a T-Slot with Rotation Point 5 is a T-Spin, even with a hole in it
            (1, 2) if rotation_point == 5 => Some(TSpin::Full),
            (1, 2) => Some(TSpin::Mini),
            _ => None,
        }
    }

    /// Whether `pos` holds a Block or is outside of the Matrix, i.e. is a Surface
    fn is_occupied(&self, pos: Pos) -> bool {
        if pos.x < 0 || pos.x >= MATRIX_WIDTH as i8 || pos.y < 0 {
            return true;
        }
        pos.y < MATRIX_HEIGHT as i8 && self.at_pos(pos) != Entity::PLACEHOLDER
    }

    pub fn at_pos(&self, pos: Pos) -> Entity {
        if pos.x < 0 || pos.y < 0 {
            return Entity::PLACEHOLDER;
        }

        self.board[pos.to_index()]
    }

    pub fn insert(&mut self, pos: Pos, id: Entity) {
        info!("Inserting block at {pos}");
        self.board[pos.to_index()] = id;
    }

    pub fn full_lines(&self) -> Vec<usize> {
        self.board
            .as_chunks::<{ MATRIX_WIDTH as usize }>()
            .0
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                line.iter()
                    .all(|e| *e != Entity::PLACEHOLDER)
                    .then_some(idx)
            })
            .collect()
    }

    pub fn line(&self, line: usize) -> &[Entity] {
        &self.board[line * MATRIX_WIDTH as usize..][..MATRIX_WIDTH as usize]
    }

    pub fn entities_to_delete(&self) -> impl Iterator<Item = Entity> + '_ {
        self.full_lines()
            .into_iter()
            .flat_map(|line| self.line(line).iter().copied())
    }

    pub fn iter_non_empty(&self) -> impl Iterator<Item = (Pos, Entity)> + '_ {
        self.board.iter().enumerate().filter_map(|(index, entity)| {
            if *entity != Entity::PLACEHOLDER {
                Some((Pos::from_index(index), *entity))
            } else {
                None
            }
        })
    }

    pub fn delete_line(&mut self, line: usize) {
        // Shift everything down by 1 row
        self.board.copy_within(
            ((line + 1) * MATRIX_WIDTH as usize)..,
            line * MATRIX_WIDTH as usize,
        );
        // Clear out the top-most row
        self.board[((MATRIX_HEIGHT as usize - 1) * MATRIX_WIDTH as usize)..]
            .fill(Entity::PLACEHOLDER);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A matrix with blocks at `blocks`
    fn matrix_with(blocks: &[(i8, i8)]) -> Matrix {
        let mut world = World::new();
        let mut matrix = Matrix::new();
        for &pos in blocks {
            matrix.insert(pos.into(), world.spawn_empty().id());
        }
        matrix
    }

    fn t_facing(facing: Facing) -> Tetrimino {
        Tetrimino {
            kind: TetriminoKind::T,
            facing,
        }
    }

    #[test]
    fn both_front_corners_and_a_back_corner_is_a_t_spin() {
        let matrix = matrix_with(&[(3, 0), (5, 0), (3, 2)]);
        let piece = t_facing(Facing::South);
        assert_eq!(matrix.t_spin(&piece, &Pos::new(4, 1), 1), Some(TSpin::Full));
    }

    #[test]
    fn both_back_corners_and_a_front_corner_is_a_mini_t_spin() {
        // The back corners are the wall
        let matrix = matrix_with(&[(1, 0)]);
        let piece = t_facing(Facing::East);
        assert_eq!(matrix.t_spin(&piece, &Pos::new(0, 1), 1), Some(TSpin::Mini));
    }

    #[test]
    fn the_floor_counts_as_occupied_corners() {
        let matrix = matrix_with(&[(5, 1)]);
        let piece = t_facing(Facing::North);
        assert_eq!(matrix.t_spin(&piece, &Pos::new(4, 0), 1), Some(TSpin::Mini));
    }

    #[test]
    fn rotation_point_5_into_a_t_slot_is_a_t_spin() {
        // An overhang blocks Rotation Points 1 to 4, and only one front corner of the T-Slot below
        // is filled
        let matrix = matrix_with(&[(4, 3), (3, 5), (2, 3), (2, 1)]);
        let piece = t_facing(Facing::North);

        let (rotated, pos, point) = matrix
            .rotate(&piece, &Pos::new(4, 4), Rotation::Clockwise)
            .unwrap();

        assert_eq!(
            (rotated.facing, pos, point),
            (Facing::East, Pos::new(3, 2), 5)
        );
        assert_eq!(matrix.t_spin(&rotated, &pos, point), Some(TSpin::Full));
        assert_eq!(matrix.t_spin(&rotated, &pos, 1), Some(TSpin::Mini));
    }

    #[test]
    fn unobstructed_rotation_uses_rotation_point_1() {
        let matrix = matrix_with(&[]);
        let piece = t_facing(Facing::North);

        let rotated = matrix.rotate(&piece, &Pos::new(4, 4), Rotation::CounterClockwise);

        assert_eq!(rotated, Some((t_facing(Facing::West), Pos::new(4, 4), 1)));
    }

    #[test]
    fn rotating_against_a_wall_kicks_off_it() {
        let matrix = matrix_with(&[]);
        let piece = t_facing(Facing::East);

        let rotated = matrix.rotate(&piece, &Pos::new(0, 4), Rotation::CounterClockwise);

        assert_eq!(rotated, Some((t_facing(Facing::North), Pos::new(1, 4), 2)));
    }

    #[test]
    fn i_tetrimino_kicks_off_the_floor() {
        let matrix = matrix_with(&[]);
        let piece = Tetrimino::from(TetriminoKind::I);

        let (rotated, pos, point) = matrix
            .rotate(&piece, &Pos::new(4, 0), Rotation::Clockwise)
            .unwrap();

        assert_eq!(
            (rotated.facing, pos, point),
            (Facing::East, Pos::new(5, 2), 5)
        );
        let cells = rotated.block_positions(&pos);
        assert!(cells.iter().all(|cell| cell.x == 6), "{cells:?}");
        assert_eq!(cells.iter().map(|cell| cell.y).min(), Some(0));
    }

    #[test]
    fn i_tetrimino_rotates_within_its_bounding_box() {
        let north = Tetrimino::from(TetriminoKind::I);
        let pos = Pos::new(4, 10);
        let cells = |piece: Tetrimino| {
            let mut cells = piece.block_positions(&pos).map(|cell| (cell.x, cell.y));
            cells.sort();
            cells
        };

        // The guideline's 4x4 box spans x 3..=6 and y 8..=11
        assert_eq!(cells(north), [(3, 10), (4, 10), (5, 10), (6, 10)]);
        assert_eq!(
            cells(north.rotated_cw()),
            [(5, 8), (5, 9), (5, 10), (5, 11)]
        );
        assert_eq!(
            cells(north.rotated_cw().rotated_cw()),
            [(3, 9), (4, 9), (5, 9), (6, 9)]
        );
        assert_eq!(
            cells(north.rotated_ccw()),
            [(4, 8), (4, 9), (4, 10), (4, 11)]
        );
    }

    #[test]
    fn rotation_fails_when_every_rotation_point_collides() {
        // A one cell wide well, so the I-Tetrimino can't lie flat
        let walls: Vec<_> = (0..4).flat_map(|y| [(3, y), (5, y)]).collect();
        let matrix = matrix_with(&walls);
        let piece = Tetrimino::from(TetriminoKind::I).rotated_cw();

        let rotated = matrix.rotate(&piece, &Pos::new(3, 2), Rotation::Clockwise);

        assert_eq!(rotated, None);
    }

    #[test]
    fn two_corners_is_not_a_t_spin() {
        let matrix = matrix_with(&[(3, 0), (5, 0)]);
        let piece = t_facing(Facing::North);
        assert_eq!(matrix.t_spin(&piece, &Pos::new(4, 1), 1), None);
    }

    #[test]
    fn only_the_t_tetrimino_can_t_spin() {
        let matrix = matrix_with(&[(3, 0), (5, 0), (3, 2), (5, 2)]);
        let piece = Tetrimino::from(TetriminoKind::S);
        assert_eq!(matrix.t_spin(&piece, &Pos::new(4, 1), 1), None);
    }
}
