#version 450
precision highp float;

layout(location = 0) in ivec2 in_pos;
layout(location = 1) in uint in_dim;
layout(location = 2) in uint in_color;
layout(location = 3) in uint in_color2;
layout(location = 4) in uint in_border_color;
layout(location = 5) in uint round_border_gradient_srgb;
layout(location = 6) in float depth;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec2 out_uv;
layout(location = 2) out vec4 out_border_color;
layout(location = 3) out float out_border_size;
layout(location = 4) out float out_radius;
layout(location = 5) out float out_rect_aspect;
layout(location = 6) out float out_pixel_size;

layout(set = 0, binding = 0) uniform UniformParams {
  uniform uvec2 screen_resolution;
};

float srgb_to_linear(float c) {
  if (c <= 0.04045) {
    return c / 12.92;
  } else {
    return pow((c + 0.055) / 1.055, 2.4);
  }
}

void main() {
  ivec2 pos = in_pos;
  uint rect_width = in_dim & 0xffffu;
  uint rect_height = (in_dim & 0xffff0000u) >> 16u;

  out_pixel_size = 1.0 / float(rect_height);

  uint v = uint(gl_VertexIndex);

  uvec2 corner_position = uvec2(v & 1u, (v >> 1u) & 1u);
  out_uv = vec2(corner_position);

  uvec2 corner_offset = uvec2(rect_width, rect_height) * corner_position;
  pos = pos + ivec2(corner_offset);

  out_rect_aspect = float(rect_width) / float(rect_height);

  gl_Position =
      vec4(2.0 * vec2(pos) / vec2(screen_resolution) - 1.0, depth, 1.0);

  out_border_color =
      vec4(float((in_border_color & 0x00ff0000u) >> 16u) / 255.0,
           float((in_border_color & 0x0000ff00u) >> 8u) / 255.0,
           float(in_border_color & 0x000000ffu) / 255.0,
           float((in_border_color & 0xff000000u) >> 24u) / 255.0);

  out_radius = (float(round_border_gradient_srgb & 0xffu) / 255.0) / 2.0;

  float border_size = float((round_border_gradient_srgb & 0xff00u) >> 8);

  out_border_size = border_size / float(rect_height);

  uint gradient_mode = (round_border_gradient_srgb & 0x00ff0000u) >> 16;

  uint color;
  switch (gradient_mode) {
  case 1:
    // horizontal
    color = corner_position.x > 0u ? in_color2 : in_color;
    break;
  case 2:
    // vertical
    color = corner_position.y > 0u ? in_color2 : in_color;
    break;
  default: // no gradient
    color = in_color;
    break;
  }

  uint srgb = (round_border_gradient_srgb & 0xff000000u) >> 24;

  if (srgb == 0u) {
    out_color = vec4(float((color & 0x00ff0000u) >> 16u) / 255.0,
                     float((color & 0x0000ff00u) >> 8u) / 255.0,
                     float(color & 0x000000ffu) / 255.0,
                     float((color & 0xff000000u) >> 24u) / 255.0);
  } else {
    out_color =
        vec4(srgb_to_linear(float((color & 0x00ff0000u) >> 16u) / 255.0),
             srgb_to_linear(float((color & 0x0000ff00u) >> 8u) / 255.0),
             srgb_to_linear(float(color & 0x000000ffu) / 255.0),
             float((color & 0xff000000u) >> 24u) / 255.0);
  }
}