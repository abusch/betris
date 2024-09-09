use bevy::prelude::*;
use bevy::{color::palettes, ecs::system::lifetimeless::SRes};
use iyes_perf_ui::{entry::PerfUiEntry, prelude::PerfUiRoot, PerfUiAppExt};

use crate::{screen::Screen, AppSet};

use super::{Phase, SCALE};

pub fn plugin(app: &mut App) {
    app.add_perf_ui_simple_entry::<PerfUiPhase>()
        .add_systems(
            Update,
            setup
                .run_if(in_state(Screen::Gameplay))
                .in_set(AppSet::Update),
        )
        .add_systems(
            Update,
            (debug_grid)
                .run_if(in_state(Screen::Gameplay))
                .in_set(AppSet::Update),
        );
}

fn setup(mut commands: Commands, root: Query<Entity, Added<PerfUiRoot>>) {
    if let Ok(root) = root.get_single() {
        info!("Adding phase debug perf ui entry");
        commands.entity(root).insert(PerfUiPhase);
    }
}

fn debug_grid(mut gizmos: Gizmos) {
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

#[derive(Component)]
pub struct PerfUiPhase;

impl PerfUiEntry for PerfUiPhase {
    type SystemParam = Option<SRes<State<Phase>>>;

    type Value = String;

    fn label(&self) -> &str {
        "Phase"
    }

    fn sort_key(&self) -> i32 {
        -1
    }

    fn update_value(
        &self,
        state: &mut <Self::SystemParam as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        state.as_ref().map(|phase| phase.to_string())
    }
}
