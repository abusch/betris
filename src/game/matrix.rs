use bevy::prelude::*;

use crate::model::{Pos, Tetrimino};

use super::{MATRIX_HEIGHT, MATRIX_WIDTH};

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
            .chunks_exact(MATRIX_WIDTH as usize)
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
