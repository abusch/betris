use bevy::color::palettes;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;

use crate::{AppSystems, screen::Screen};

use super::SCALE;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (debug_grid)
            .run_if(in_state(Screen::Gameplay).and_then(input_toggle_active(false, KeyCode::KeyG)))
            .in_set(AppSystems::Update),
    );
}

fn debug_grid(mut gizmos: Gizmos) {
    gizmos
        .grid_2d(
            Isometry2d::new(Vec2::new(-5.0 * SCALE, 0.0), Rot2::radians(0.0)),
            UVec2::new(10, 22),
            Vec2::new(SCALE, SCALE),
            palettes::css::HOT_PINK.with_alpha(0.5),
        )
        .outer_edges();
}
