pub mod vert_rect {
	vulkano_shaders::shader! {
			ty: "vertex",
			src: r"#version 310 es
            precision highp float;

            layout (location = 0) in ivec2 in_pos;
            layout (location = 1) in vec2 in_uv;
            layout (location = 2) in uint in_dim;
            layout (location = 3) in uint in_color;
            layout (location = 4) in float corner_radius;
            layout (location = 5) in uint content_type_with_srgb;
            layout (location = 6) in float depth;

            layout (location = 0) out vec4 out_color;
            layout (location = 1) out vec2 out_radius;

            layout (set = 2, binding = 0) uniform UniformParams {
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
							out_uv = in_uv;

						  ivec2 pos = in_pos;
							uint width = in_dim & 0xffffu;
							uint height = (in_dim & 0xffff0000u) >> 16u;

							uint v = uint(gl_VertexIndex);

							uvec2 corner_position = uvec2(v & 1u, (v >> 1u) & 1u);

							uvec2 corner_offset = uvec2(width, height) * corner_position;

							pos = pos + ivec2(corner_offset);

              gl_Position = vec4(
								2.0 * vec2(pos) / vec2(screen_resolution) - 1.0,
								depth,
								1.0
							);

							gl_Position.y *= -1.0;

							content_type = content_type_with_srgb & 0xffffu;
              uint srgb = (content_type_with_srgb & 0xffff0000u) >> 16u;

              if (srgb == 0u) {
							  out_color = vec4(
									float((in_color & 0x00ff0000u) >> 16u) / 255.0,
									float((in_color & 0x0000ff00u) >> 8u) / 255.0,
									float(in_color & 0x000000ffu) / 255.0,
									float((in_color & 0xff000000u) >> 24u) / 255.0
								);
							} else {
							  out_color = vec4(
									srgb_to_linear(float((in_color & 0x00ff0000u) >> 16u) / 255.0),
									srgb_to_linear(float((in_color & 0x0000ff00u) >> 8u) / 255.0),
									srgb_to_linear(float(in_color & 0x000000ffu) / 255.0),
									float((in_color & 0xff000000u) >> 24u) / 255.0
								);
							}

							uvec2 dim = uvec2(0, 0);
							if (content_type == 0u) {
							  dim = uvec2(textureSize(color_atlas, 0));
							} else {
							  dim = uvec2(textureSize(mask_atlas, 0));
							}
            }
        ",
	}
}

pub mod frag_rect {
	vulkano_shaders::shader! {
			ty: "fragment",
			src: r"#version 310 es
            precision highp float;

            layout (location = 0) in vec4 in_color;
            layout (location = 1) in float in_radius;

            layout (location = 0) out vec4 out_color;

            void main()
            {
                out_color.r = corner_radius.r;
                out_color = in_color;

                vec2 uv_circ = ((1. - corner_radius) - (abs(in_uv + vec2(-0.5)) * 2.))/corner_radius;
                float dist = length(uv_circ);

                out_color.a = mix(out_color.a, 0.,
                        float(dist > 1.)
                        * float(uv_circ.x < 0.)
                        * float(uv_circ.y < 0.));
            }
        ",
	}
}