use bevy::prelude::*;

use crate::screen::Screen;

use super::score::Score;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), setup)
        .add_systems(Update, update);
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let text_style = TextStyle {
        font: assets.load("fonts/BungeeSpice-Regular.ttf"),
        font_size: 40.0,
        color: Color::BLACK,
    };

    commands.spawn((
        Name::new("Score"),
        TextBundle::from_sections([
            TextSection::new("Score: ", text_style.clone()),
            TextSection::new("", text_style),
        ])
        .with_no_wrap()
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
        ScoreText,
    ));
}

#[derive(Component)]
struct ScoreText;

fn update(score: Res<Score>, mut text: Query<&mut Text, With<ScoreText>>) {
    if let Ok(mut score_text) = text.get_single_mut() {
        score_text.sections[1].value = score.formatted();
    }
}
