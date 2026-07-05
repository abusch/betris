use bevy::{prelude::*, text::FontSourceTemplate};

use crate::screen::Screen;

use super::score::Score;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), setup)
        .add_systems(Update, update);
}

fn setup(mut commands: Commands) {
    let font_name = "fonts/BungeeSpice-Regular.ttf";
    commands.spawn_scene(bsn! {
        #Score
        Text("Score: ")
        TextLayout::no_wrap()
        TextFont {
            font: FontSourceTemplate::Handle(font_name),
            font_size: FontSize::Px(40.0)
        }
        TextColor(Color::WHITE)
        Node {
            position_type: PositionType::Absolute,
            top: px(5.0),
            left: px(5.0),
        }
        Children [(
            TextSpan("")
            ScoreText
            TextFont {
                font: FontSourceTemplate::Handle(font_name),
                font_size: FontSize::Px(40.0)
            }
        )]
    });
}

#[derive(Default, Copy, Clone, Component)]
struct ScoreText;

fn update(score: Res<Score>, mut text: Query<&mut TextSpan, With<ScoreText>>) {
    if let Ok(mut score_text) = text.single_mut() {
        score_text.0 = score.formatted();
    }
}
