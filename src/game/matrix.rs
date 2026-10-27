use bevy::prelude::*;

use crate::model::{Facing, Pos, Tetrimino, TetriminoKind};

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

    /// Recognise a T-Spin by the T-Tetrimino locking down at `pos`, using the cells diagonally
    /// adjacent to its center (see section 9.1 of the guideline). The caller must check that the
    /// piece's last move was a rotation.
    ///
    /// This doesn't implement the Rotation Point 5 rule, since rotations don't kick (yet).
    pub fn t_spin(&self, piece: &Tetrimino, pos: &Pos) -> Option<TSpin> {
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
        assert_eq!(matrix.t_spin(&piece, &Pos::new(4, 1)), Some(TSpin::Full));
    }

    #[test]
    fn both_back_corners_and_a_front_corner_is_a_mini_t_spin() {
        // The back corners are the wall
        let matrix = matrix_with(&[(1, 0)]);
        let piece = t_facing(Facing::East);
        assert_eq!(matrix.t_spin(&piece, &Pos::new(0, 1)), Some(TSpin::Mini));
    }

    #[test]
    fn the_floor_counts_as_occupied_corners() {
        let matrix = matrix_with(&[(5, 1)]);
        let piece = t_facing(Facing::North);
        assert_eq!(matrix.t_spin(&piece, &Pos::new(4, 0)), Some(TSpin::Mini));
    }

    #[test]
    fn two_corners_is_not_a_t_spin() {
        let matrix = matrix_with(&[(3, 0), (5, 0)]);
        let piece = t_facing(Facing::North);
        assert_eq!(matrix.t_spin(&piece, &Pos::new(4, 1)), None);
    }

    #[test]
    fn only_the_t_tetrimino_can_t_spin() {
        let matrix = matrix_with(&[(3, 0), (5, 0), (3, 2), (5, 2)]);
        let piece = Tetrimino::from(TetriminoKind::S);
        assert_eq!(matrix.t_spin(&piece, &Pos::new(4, 1)), None);
    }
}
