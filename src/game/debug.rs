use bevy::ecs::system::lifetimeless::SRes;
use bevy::prelude::*;
use iyes_perf_ui::{entry::PerfUiEntry, prelude::PerfUiRoot, PerfUiAppExt};

use crate::{screen::Screen, AppSet};

use super::Phase;

pub fn plugin(app: &mut App) {
    app.add_perf_ui_simple_entry::<PerfUiPhase>().add_systems(
        Update,
        setup
            .run_if(in_state(Screen::Playing))
            .in_set(AppSet::Update),
    );
}

fn setup(mut commands: Commands, root: Query<Entity, Added<PerfUiRoot>>) {
    if let Ok(root) = root.get_single() {
        info!("Adding phase debug perf ui entry");
        commands.entity(root).insert(PerfUiPhase);
    }
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
