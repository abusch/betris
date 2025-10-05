use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin)
        .add_input_context::<InGame>()
        // .add_observer(binding)
        .add_observer(toggle_pause);
}

pub fn bindings(commands: &mut Commands) {
    commands.spawn((
        InGame,
        actions!(InGame[
            (Action::<Left>::new(), bindings![KeyCode::ArrowLeft],),
            (Action::<Right>::new(), bindings![KeyCode::ArrowRight],),
            (Action::<RotateLeft>::new(), bindings![KeyCode::KeyZ, KeyCode::ArrowUp],),
            (Action::<RotateRight>::new(), bindings![KeyCode::KeyX],),
            (Action::<SoftDrop>::new(), bindings![KeyCode::ArrowDown],),
            (Action::<HardDrop>::new(), bindings![KeyCode::Space],),
            (Action::<Pause>::new(), bindings![KeyCode::KeyP],),
        ]),
    ));
}

#[derive(Debug, Default, Component)]
pub struct InGame;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct Left;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct Right;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct RotateLeft;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct RotateRight;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct SoftDrop;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct HardDrop;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct Pause;

fn toggle_pause(_trigger: On<Start<Pause>>, mut time: ResMut<Time<Virtual>>) {
    if time.is_paused() {
        time.unpause();
    } else {
        time.pause();
    }
}
