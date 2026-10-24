use bevy::{prelude::*, text::FontSourceTemplate};

use crate::screen::Screen;

use super::{Phase, score::Score};

const FONT: &str = "fonts/BungeeSpice-Regular.ttf";

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), setup)
        .add_systems(OnEnter(Phase::GameOver), show_game_over)
        .add_systems(Update, update.run_if(in_state(Screen::Gameplay)));
}

/// The game's font at the given size
pub fn game_font(size: f32) -> impl Scene {
    bsn! {
        TextFont {
            font: FontSourceTemplate::Handle(FONT),
            font_size: {FontSize::Px(size)}
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #Score
        DespawnOnExit<Screen>(Screen::Gameplay)
        Text("Score: ")
        TextLayout::no_wrap()
        @game_font(40.0)
        TextColor(Color::WHITE)
        Node {
            position_type: PositionType::Absolute,
            top: px(5.0),
            left: px(5.0),
        }
        Children [
            TextSpan("")
            ScoreText
            @game_font(40.0)
        ]
    });
}

#[derive(Default, Copy, Clone, Component)]
struct ScoreText;

fn update(score: Res<Score>, mut text: Query<&mut TextSpan, With<ScoreText>>) {
    if let Ok(mut score_text) = text.single_mut() {
        score_text.0 = score.formatted();
    }
}

fn show_game_over(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #GameOver
        DespawnOnExit<Phase>(Phase::GameOver)
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor({Color::BLACK.with_alpha(0.6)})
        Children [
            Text("Game Over")
            @game_font(78.0)
            TextColor(Color::WHITE)
            --
            Text("Press Enter to play again")
            @game_font(30.0)
            TextColor(Color::WHITE)
        ]
    });
}
