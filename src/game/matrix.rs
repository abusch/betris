use bevy::prelude::*;

use crate::{pieces::Piece, pos::Pos};

use super::{MATRIX_HEIGHT, MATRIX_WIDTH};

#[derive(Debug, Clone, Reflect)]
pub struct Matrix {
    pub root_entity: Entity,
    pub board: [Entity; MATRIX_WIDTH * MATRIX_HEIGHT],
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
            board: [Entity::PLACEHOLDER; MATRIX_WIDTH * MATRIX_HEIGHT],
        }
    }

    pub fn is_pos_valid(&self, piece: &Piece, pos: Pos) -> bool {
        for p in piece.block_positions(pos) {
            if p.x < 0
                || p.x >= (MATRIX_WIDTH as isize)
                || p.y < 0
                || self.at_pos(p) != Entity::PLACEHOLDER
            {
                return false;
            }
        }
        true
    }

    pub fn is_on_surface(&self, piece: &Piece, pos: Pos) -> bool {
        !self.is_pos_valid(piece, pos.down())
    }

    pub fn lowest_valid_pos(&self, piece: &Piece, pos: Pos) -> Pos {
        let mut p = pos;

        while !self.is_on_surface(piece, p) {
            p = p.down();
        }
        p
    }

    pub fn at(&self, x: usize, y: usize) -> Entity {
        self.board[y * MATRIX_WIDTH + x]
    }

    pub fn at_mut(&mut self, x: usize, y: usize) -> &mut Entity {
        &mut self.board[y * MATRIX_WIDTH + x]
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
            .chunks_exact(MATRIX_WIDTH)
            .enumerate()
            .filter_map(|(idx, line)| {
                line.iter()
                    .all(|e| *e != Entity::PLACEHOLDER)
                    .then_some(idx)
            })
            .collect()
    }

    pub fn line(&self, line: usize) -> &[Entity] {
        &self.board[line * MATRIX_WIDTH..][..MATRIX_WIDTH]
    }

    pub fn entities_to_delete(&self) -> impl Iterator<Item = Entity> + '_ {
        self.full_lines()
            .into_iter()
            .flat_map(|line| self.line(line).iter().copied())
    }

    pub fn delete_line(&mut self, line: usize) {
        // Shift everything down by 1 row
        self.board
            .copy_within(((line + 1) * MATRIX_WIDTH).., line * MATRIX_WIDTH);
        // Clear out the top-most row
        self.board[((MATRIX_HEIGHT - 1) * MATRIX_WIDTH)..].fill(Entity::PLACEHOLDER);
        // for x in 0..MATRIX_WIDTH {
        //     for y in line..(MATRIX_HEIGHT - 1) {
        //         *self.at_mut(x, y) = self.at(x, y + 1);
        //     }
        //     *self.at_mut(x, MATRIX_HEIGHT - 1) = Entity::PLACEHOLDER;
        // }
    }

    pub fn remove(&mut self, pos: Pos) {
        info!("Block removed from matrix at {pos}");
        self.board[pos.to_index()] = Entity::PLACEHOLDER;
    }
}
