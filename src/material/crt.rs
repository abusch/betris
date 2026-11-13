use bevy::{
    core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
    prelude::*,
    render::{RenderApp, extract_component::ExtractComponent, render_resource::ShaderType},
    shader::ShaderRef,
};

use crate::camera::RESOLUTION;

pub fn plugin(app: &mut App) {
    app.add_plugins(FullscreenMaterialPlugin::<Crt>::default());
}

#[derive(Component, ExtractComponent, Clone, Copy, ShaderType)]
#[extract_app(RenderApp)]
pub struct Crt {
    pub low_resolution: UVec2,
    pub mask_strength: f32,
}

impl Default for Crt {
    fn default() -> Self {
        Self {
            // One line per row and column of game pixels
            low_resolution: RESOLUTION,
            mask_strength: 1.0,
        }
    }
}

impl FullscreenMaterial for Crt {
    fn fragment_shader() -> ShaderRef {
        "shaders/crt.wesl".into()
    }

    fn schedule() -> impl bevy::ecs::schedule::ScheduleLabel + Clone {
        bevy::core_pipeline::Core2d
    }

    fn schedule_configs(
        system: bevy::ecs::schedule::ScheduleConfigs<bevy::ecs::system::BoxedSystem>,
    ) -> bevy::ecs::schedule::ScheduleConfigs<bevy::ecs::system::BoxedSystem> {
        // Post-processing passes swap the view's main textures when they run, but their commands
        // are submitted in the schedule's order. Unordered passes can run in a different order
        // from one frame to the next, and then read the wrong texture, e.g. losing the bloom on
        // some frames, which flickers. Running before bloom puts this pass in a fixed position:
        // main pass, scanlines, bloom, effect stack (lens distortion, ...), tonemapping. Bloom
        // then also smooths the scanlines of bright colors a little, and lens distortion curves
        // them, like a CRT's glass.
        system
            .in_set(bevy::core_pipeline::Core2dSystems::PostProcess)
            .before(bevy::post_process::bloom::bloom)
    }
}
