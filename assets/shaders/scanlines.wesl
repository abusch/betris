import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput;
import bevy_render::maths::PI;

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct Scanlines {
  num_lines: u32,
  num_cols: u32,
  intensity: f32,
}

@group(0) @binding(2) var<uniform> settings: Scanlines;

/// Cheap scanline effect.
@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
  let scanline = abs(
    sin(in.uv.y * f32(settings.num_lines) * PI) *
    sin(in.uv.x * f32(settings.num_cols) * PI)
  ) * (1.0 - settings.intensity) + settings.intensity;
  var pixel = textureSample(screen_texture, texture_sampler, in.uv);

  pixel.r *= scanline;
  pixel.g *= scanline;
  pixel.b *= scanline;

  return pixel;
}
