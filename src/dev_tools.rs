use bevy::prelude::*;

use crate::AppSet;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, close_on_esc.in_set(AppSet::RecordInput));
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
