use bevy::prelude::*;

use crate::{pieces::Piece, pos::Pos};

use super::{MATRIX_HEIGHT, MATRIX_WIDTH};

#[derive(Debug, Clone)]
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
                || self.at(p) != Entity::PLACEHOLDER
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

    pub fn at(&self, pos: Pos) -> Entity {
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
}
