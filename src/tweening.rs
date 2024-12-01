use bevy::color::{Color, Mix};
use bevy_tween::{component_tween_system, prelude::*};
use bevy_vector_shapes::shapes::ShapeFill;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(DefaultTweenPlugins)
        .add_tween_systems(component_tween_system::<ShapeColor>());
}

/// Custom interpolator for the color of a [`bevy_vector_shapes::shapes::ShapeBundle`]
pub struct ShapeColor {
    start: Color,
    end: Color,
}

impl Interpolator for ShapeColor {
    type Item = ShapeFill;

    fn interpolate(&self, item: &mut Self::Item, value: f32) {
        item.color = self.start.mix(&self.end, value);
    }
}

pub fn shape_color(start: Color, end: Color) -> ShapeColor {
    ShapeColor { start, end }
}

pub fn shape_color_to(to: Color) -> impl Fn(&mut Color) -> ShapeColor {
    move |state| {
        let start = *state;
        let end = to;
        *state = to;
        shape_color(start, end)
    }
}
