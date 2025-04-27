use bevy::prelude::*;

use crate::screen::Screen;

use super::score::Score;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), setup)
        .add_systems(Update, update);
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let text_font = TextFont {
        font: assets.load("fonts/BungeeSpice-Regular.ttf"),
        font_size: 40.0,
        ..default()
    };
    let text_color = TextColor(Color::WHITE);

    commands
        .spawn((
            Name::new("Score"),
            Text::new("Score: "),
            text_font.clone(),
            text_color,
            TextLayout::new_with_no_wrap(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
        ))
        .with_children(|children| {
            children.spawn((
                TextSpan("".into()),
                text_font.clone(),
                text_color,
                ScoreText,
            ));
        });
}

#[derive(Component)]
struct ScoreText;

fn update(score: Res<Score>, mut text: Query<&mut TextSpan, With<ScoreText>>) {
    if let Ok(mut score_text) = text.single_mut() {
        score_text.0 = score.formatted();
    }
}
