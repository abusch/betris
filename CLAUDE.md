# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Betris is a Tetris clone built with the Bevy game engine in Rust. The game follows a modular architecture with clear separation between game logic, rendering, and input handling.

## Development Commands

- `cargo run` - Run the game in debug mode
- `cargo run --release` - Run the game in release mode (optimized)
- `cargo build` - Build the project
- `cargo test` - Run tests
- `cargo clippy` - Run linting
- `cargo fmt` - Format code

### Feature Flags

- `dev` - Enables development features like dynamic linking for faster compile times
- `dev_native` - Includes `dev` plus file watching for hot reloading
- Default feature set includes `dev_native`

## Architecture

### Core Structure

The project is organized into several key modules:

- **`src/main.rs`** - Entry point that initializes the Bevy app with `AppPlugin`
- **`src/lib.rs`** - Main app plugin that sets up the game systems and plugins
- **`src/game/`** - Core game logic including the Tetris gameplay mechanics
- **`src/model/`** - Data structures for tetriminos, positions, and game state
- **`src/screen/`** - Screen management (splash, gameplay)
- **`src/tweening.rs`** - Animation and tweening utilities

### Game State Management

The game uses Bevy's state system with two main state types:

1. **Screen states** (`src/screen/mod.rs`):
   - `Splash` - Initial screen
   - `Gameplay` - Main game screen

2. **Game phases** (`src/game/mod.rs`):
   - `Generation` - Spawning new pieces
   - `Falling` - Piece falling down
   - `Lock` - Piece locking into place
   - `Pattern` - Detecting line clears
   - `Animate` - Animating line clears
   - `Eliminate` - Removing cleared lines

### System Organization

Systems are organized into three main categories (defined in `AppSystems`):
- `TickTimers` - Handle timing and game progression
- `RecordInput` - Process player input
- `Update` - General game updates

### Key Components

- **Matrix** (`src/game/matrix.rs`) - Represents the game board as a 10x40 grid
- **Tetrimino** (`src/model/tetrimino.rs`) - Piece definitions and rotations
- **Positioned** (`src/game/spawners/mod.rs`) - Grid-based positioning
- **Block** (`src/game/mod.rs`) - Static blocks committed to the matrix

### Input System

Uses `bevy_enhanced_input` for handling player controls:
- Rotation (left/right)
- Movement (left/right)
- Soft drop and hard drop
- Input actions defined in `src/game/input.rs`

### Rendering

- Uses `bevy_vector_shapes` for shape rendering
- 2D camera setup with HDR and bloom effects
- Scale factor of 20.0 for grid-to-world coordinate conversion
- Custom tweening system for animations

## Development Notes

- The project uses Bevy 0.16.0 with dynamic linking enabled in dev mode for faster compilation
- Code includes comprehensive logging with `info!`, `warn!`, and `error!` macros
- Debug features are conditionally compiled with `#[cfg(feature = "dev")]`
- The codebase follows Bevy's ECS patterns with systems, components, and resources