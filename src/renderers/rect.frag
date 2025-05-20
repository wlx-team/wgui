#version 450
precision highp float;

layout(location = 0) in vec4 in_color;
layout(location = 1) in vec4 in_color2;
layout(location = 2) in vec2 in_uv;
layout(location = 3) in vec4 in_border_color;
layout(location = 4) in float in_border_size;
layout(location = 5) in float in_radius;
layout(location = 6) in float in_rect_aspect;
layout(location = 7) in float in_pixel_size;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform UniformParams {
  uniform uvec2 screen_resolution;
};

void main() {
  vec2 rect_dim = vec2(in_rect_aspect, 1.0);
  vec2 center = rect_dim / 2.0;
  vec2 coords = in_uv * rect_dim;

  vec2 sdf_rect_dim = center - vec2(in_radius);
  float sdf = length(max(abs(coords - center), sdf_rect_dim) - sdf_rect_dim) -
              in_radius;

  vec4 color = mix(in_color, in_color2, min(length((in_uv - vec2(0.5)) * 2.0), 1.0));

  if (in_border_size < in_radius) {
    // rounded border
    float f = smoothstep(-in_pixel_size, 0.0, sdf + in_border_size) *
              in_border_color.a;
    out_color = mix(color, in_border_color, f);
  } else {
    // square border
    vec2 a = abs(in_uv - 0.5);
    float aa = 0.5 - in_border_size / in_rect_aspect;
    float bb = 0.5 - in_border_size;
    out_color = (a.x > aa || a.y > bb) ? in_border_color : color;
  }

  if (in_radius > 0.0) {
    // rounding cutout alpha
    out_color.a *= 1.0 - smoothstep(-in_pixel_size, 0.0, sdf);
  }
}
