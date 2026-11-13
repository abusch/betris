import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;
import bevy_render::maths::PI;

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct Crt {
  low_resolution: vec2<u32>,
  mask_strength: f32,
}

@group(0) @binding(2) var<uniform> settings: Crt;

/// Cheap scanline effect.
@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
  // Original color
  var pixel = textureSample(screen_texture, texture_sampler, in.uv);

  // Apply phosphor grid effect
  pixel = apply_phosphor_grid(pixel, in.uv);

  // Apply scanline effect
  var scanline = sin(in.uv.y * f32(settings.low_resolution.y) * PI);
  scanline = mix(0.6, 1.0, (scanline + 1.0) * 0.5);
  pixel.r *= scanline;
  pixel.g *= scanline;
  pixel.b *= scanline;

  return pixel;
}

fn apply_phosphor_grid(color: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
  let pixel_x = fract(uv.x * f32(settings.low_resolution.x)) * 3.0;
  var mask = vec4(0.0, 0.0, 0.0, 1.0);

  mask.r = clamp(1.4 - abs(pixel_x - 0.5), 0.3, 1.0);
  mask.g = clamp(1.4 - abs(pixel_x - 1.5), 0.3, 1.0);
  mask.b = clamp(1.4 - abs(pixel_x - 2.5), 0.3, 1.0);

  return color * mix(vec4(1.0, 1.0, 1.0, 1.0), mask, settings.mask_strength);
}
