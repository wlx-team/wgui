#version 310 es
#extension GL_GOOGLE_include_directive : enable

precision highp float;

layout(location = 0) in vec4 in_color;
layout(location = 1) in vec2 in_uv;
layout(location = 2) flat in vec4 in_uv_range;
layout(location = 3) flat in vec2 in_texel_size;
layout(location = 4) flat in uint content_type;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D color_atlas;
layout(set = 1, binding = 0) uniform sampler2D mask_atlas;

vec4 clamped_bilinear(sampler2D tex0) 
{
  vec2 uv0 = clamp(in_uv - in_texel_size, in_uv_range.xy, in_uv_range.zw);
  vec2 uv1 = clamp(in_uv + vec2(in_texel_size.x, -in_texel_size.y), in_uv_range.xy, in_uv_range.zw);
  vec2 uv2 = clamp(in_uv + vec2(-in_texel_size.x, in_texel_size.y), in_uv_range.xy, in_uv_range.zw);
  vec2 uv3 = clamp(in_uv + in_texel_size, in_uv_range.xy, in_uv_range.zw);

  vec4 samp0 = texture(tex0, uv0);
  vec4 samp1 = texture(tex0, uv1);
  vec4 samp2 = texture(tex0, uv2);
  vec4 samp3 = texture(tex0, uv3);

  vec2 uv_texel_norm = vec2(0.5, 0.5); //fract(in_uv / in_texel_size);

  return mix(
    mix(samp0, samp1, uv_texel_norm.x),
    mix(samp2, samp3, uv_texel_norm.x),
    uv_texel_norm.y
  );
}

void main() {
  if (content_type == 0u) {
    out_color = clamped_bilinear(color_atlas);
  } else {
    out_color.rgb = in_color.rgb;
    out_color.a = in_color.a * clamped_bilinear(mask_atlas).r;
  }
}
