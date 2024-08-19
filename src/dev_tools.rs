use bevy::{dev_tools::states::log_transitions, prelude::*};

use crate::{game::Phase, screen::Screen, AppSet};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            close_on_esc.in_set(AppSet::RecordInput),
            log_transitions::<Screen>,
            log_transitions::<Phase>,
        ),
    );
}

fn close_on_esc(
    mut commands: Commands,
    windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (id, win) in windows.iter() {
        if win.focused && input.just_pressed(KeyCode::Escape) {
            commands.entity(id).despawn();
        }
    }
}
