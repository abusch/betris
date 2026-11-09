//! The game is rendered at a low resolution, then upscaled to the window without any filtering for
//! a retro look. The CRT-style post-processing and the debug UI are drawn at the window's
//! resolution.

use bevy::{
    camera::{Hdr, RenderTarget, ScalingMode, visibility::RenderLayers},
    core_pipeline::tonemapping::Tonemapping,
    image::ImageSampler,
    post_process::{
        bloom::Bloom,
        effect_stack::{ChromaticAberration, LensDistortion, Vignette},
    },
    prelude::*,
    render::{
        render_resource::TextureFormat,
        view::{ColorGrading, ColorGradingGlobal, DebandDither},
    },
};

use crate::material::scanlines::Scanlines;

/// The resolution the game is rendered at
pub const RESOLUTION: UVec2 = UVec2::new(540, 360);
/// How many window pixels each game pixel covers, on each axis
const PIXEL_SCALE: u32 = 2;
/// The size of the window, in logical pixels
pub const WINDOW_SIZE: UVec2 = UVec2::new(RESOLUTION.x * PIXEL_SCALE, RESOLUTION.y * PIXEL_SCALE);
/// Only the window camera sees this layer, so that it doesn't render the game a second time
const WINDOW_LAYER: usize = 1;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_observer(target_game_camera);
}

/// Marker component for the camera rendering the game at a low resolution
#[derive(Component)]
pub struct GameCamera;

/// Marker component for the root of some UI rendered at the game's resolution, as opposed to the
/// window's (e.g. the debug UI).
#[derive(Component, Default, Clone)]
pub struct GameUi;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Floating point, so that colors brighter than white survive until bloom picks them up
    let mut image =
        Image::new_target_texture(RESOLUTION.x, RESOLUTION.y, TextureFormat::Rgba16Float, None);
    image.sampler = ImageSampler::nearest();
    let image = images.add(image);

    commands.spawn((
        Name::new("Game camera"),
        GameCamera,
        Camera2d,
        Camera {
            // Render the game before it gets upscaled
            order: -1,
            ..default()
        },
        RenderTarget::Image(image.clone().into()),
        // Leave HDR colors as they are for the window camera to tone map
        Hdr,
        Tonemapping::None,
        DebandDither::Disabled,
        // Keep pixels sharp
        Msaa::Off,
    ));

    commands.spawn((
        Name::new("Game image"),
        Sprite::from_image(image),
        RenderLayers::layer(WINDOW_LAYER),
    ));

    commands.spawn((
        Name::new("Window camera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            // Fill the window with the game image
            scaling_mode: ScalingMode::AutoMin {
                min_width: RESOLUTION.x as f32,
                min_height: RESOLUTION.y as f32,
            },
            ..OrthographicProjection::default_2d()
        }),
        RenderLayers::layer(WINDOW_LAYER),
        Hdr,
        ColorGrading {
            global: ColorGradingGlobal {
                exposure: 1.2,
                ..default()
            },
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        ChromaticAberration {
            intensity: 0.01,
            ..default()
        },
        LensDistortion {
            intensity: 0.1,
            ..default()
        },
        Vignette {
            intensity: 0.6,
            radius: 0.9,
            smoothness: 2.0,
            ..default()
        },
        Scanlines::default(),
    ));
}

/// UI is rendered by the window camera by default, so point the game's UI to the game camera.
fn target_game_camera(
    add: On<Add<GameUi>>,
    camera: Single<Entity, With<GameCamera>>,
    mut commands: Commands,
) {
    commands.entity(add.entity).insert(UiTargetCamera(*camera));
}
