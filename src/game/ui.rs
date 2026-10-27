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
        #Hud
        DespawnOnExit<Screen>(Screen::Gameplay)
        Node {
            position_type: PositionType::Absolute,
            top: px(5.0),
            left: px(5.0),
            flex_direction: FlexDirection::Column,
        }
        Children [
            @counter("Score: ", Counter::Score)
            --
            @counter("Level: ", Counter::Level)
            --
            @counter("Lines: ", Counter::Lines)
        ]
    });
}

/// A line of the HUD showing the given label followed by the counter's value
fn counter(label: &'static str, counter: Counter) -> impl Scene {
    bsn! {
        Text(label)
        TextLayout::no_wrap()
        @game_font(40.0)
        TextColor(Color::WHITE)
        Children [
            TextSpan("")
            counter
            @game_font(40.0)
        ]
    }
}

/// The value displayed by a HUD text span
#[derive(Copy, Clone, Component)]
enum Counter {
    Score,
    Level,
    Lines,
}

fn update(score: Res<Score>, mut texts: Query<(&mut TextSpan, &Counter)>) {
    for (mut text, counter) in &mut texts {
        text.0 = match counter {
            Counter::Score => score.formatted(),
            Counter::Level => score.level().to_string(),
            Counter::Lines => score.lines_cleared().to_string(),
        };
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

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;
    use crate::game::score::{ScoreAction, ScoreEvent};

    #[test]
    fn counters_show_score_level_and_lines() {
        let mut world = World::new();
        let mut score = Score::default();
        score.handle_event(&ScoreEvent::LevelStart(1));
        // Not consecutive, so there is no combo bonus
        score.handle_event(&ScoreEvent::LockDown(ScoreAction::Double));
        score.handle_event(&ScoreEvent::LockDown(ScoreAction::Lock));
        score.handle_event(&ScoreEvent::LockDown(ScoreAction::Single));
        world.insert_resource(score);
        let counters = [Counter::Score, Counter::Level, Counter::Lines]
            .map(|counter| world.spawn((TextSpan::default(), counter)).id());

        world.run_system_once(update).unwrap();

        let texts = counters.map(|e| world.get::<TextSpan>(e).unwrap().0.clone());
        assert_eq!(texts, ["000400", "1", "3"]);
    }
}
