use bevy::{
    core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin},
    prelude::*,
    render::{RenderApp, extract_component::ExtractComponent, render_resource::ShaderType},
    shader::ShaderRef,
};

pub fn plugin(app: &mut App) {
    app.add_plugins(FullscreenMaterialPlugin::<Scanlines>::default());
}

#[derive(Component, ExtractComponent, Clone, Copy, ShaderType)]
#[extract_app(RenderApp)]
pub struct Scanlines {
    pub num_lines: u32,
    pub intensity: f32,
}

impl Default for Scanlines {
    fn default() -> Self {
        Self {
            num_lines: 200,
            intensity: 0.5,
        }
    }
}

impl FullscreenMaterial for Scanlines {
    fn fragment_shader() -> ShaderRef {
        "shaders/scanlines.wesl".into()
    }

    fn schedule() -> impl bevy::ecs::schedule::ScheduleLabel + Clone {
        bevy::core_pipeline::Core2d
    }

    fn schedule_configs(
        system: bevy::ecs::schedule::ScheduleConfigs<bevy::ecs::system::BoxedSystem>,
    ) -> bevy::ecs::schedule::ScheduleConfigs<bevy::ecs::system::BoxedSystem> {
        system
            .in_set(bevy::core_pipeline::Core2dSystems::PostProcess)
            .before(bevy::core_pipeline::tonemapping::tonemapping)
    }
}
