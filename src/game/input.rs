use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(EnhancedInputPlugin)
        .add_input_context::<InGame>()
        .add_observer(binding)
        .add_observer(toggle_pause);
}

pub fn binding(trigger: Trigger<Binding<InGame>>, mut game: Query<&mut Actions<InGame>>) {
    let mut actions = game.get_mut(trigger.target()).unwrap();
    actions.bind::<Left>().to(KeyCode::ArrowLeft);
    actions.bind::<Right>().to(KeyCode::ArrowRight);
    actions
        .bind::<RotateLeft>()
        .to((KeyCode::KeyZ, KeyCode::ArrowUp));
    actions.bind::<RotateRight>().to(KeyCode::KeyX);
    actions.bind::<SoftDrop>().to(KeyCode::ArrowDown);
    actions.bind::<HardDrop>().to(KeyCode::Space);
    actions.bind::<Pause>().to(KeyCode::KeyP);
}

#[derive(Debug, InputContext)]
pub struct InGame;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Left;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Right;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct RotateLeft;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct RotateRight;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct SoftDrop;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct HardDrop;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct Pause;

fn toggle_pause(_trigger: Trigger<Started<Pause>>, mut time: ResMut<Time<Virtual>>) {
    if time.is_paused() {
        time.unpause();
    } else {
        time.pause();
    }
}
