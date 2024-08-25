//! Contains core types related to the Tetris gameplay. E.g. the different tetriminos with their
//! shapes, orientation, etc.
//!
//! The code in this module should (in theory) be mostly free of any dependency on Bevy.

mod data;
mod pos;
mod tetrimino;

pub use pos::*;
pub use tetrimino::*;
