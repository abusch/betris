use bevy::{prelude::*, text::FontSourceTemplate};

use crate::screen::Screen;

use super::{Phase, score::Score};

const FONT: &str = "fonts/BungeeSpice-Regular.ttf";

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), setup)
        .add_systems(OnEnter(Phase::GameOver), show_game_over)
        .add_systems(Update, update.run_if(in_state(Screen::Gameplay)));
}

fn setup(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #Score
        DespawnOnExit<Screen>(Screen::Gameplay)
        Text("Score: ")
        TextLayout::no_wrap()
        TextFont {
            font: FontSourceTemplate::Handle(FONT),
            font_size: FontSize::Px(40.0)
        }
        TextColor(Color::WHITE)
        Node {
            position_type: PositionType::Absolute,
            top: px(5.0),
            left: px(5.0),
        }
        Children [
            TextSpan("")
            ScoreText
            TextFont {
                font: FontSourceTemplate::Handle(FONT),
                font_size: FontSize::Px(40.0)
            }
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

fn show_game_over(mut commands: Commands, assets: Res<AssetServer>) {
    let font: Handle<Font> = assets.load(FONT);
    commands.spawn((
        Name::new("Game over"),
        DespawnOnExit(Phase::GameOver),
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::BLACK.with_alpha(0.6)),
        children![
            (
                Text::new("Game Over"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(78.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ),
            (
                Text::new("Press Enter to play again"),
                TextFont {
                    font: font.into(),
                    font_size: FontSize::Px(30.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ),
        ],
    ));
}
