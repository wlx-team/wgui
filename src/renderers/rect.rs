use std::sync::Arc;

use vulkano::{
	buffer::{BufferContents, BufferUsage, Subbuffer},
	format::Format,
	pipeline::graphics::{input_assembly::PrimitiveTopology, vertex_input::Vertex},
};

use crate::{
	drawing::{Boundary, Rectangle},
	gfx::{BLEND_ALPHA, WGfx, cmd::GfxCommandBuffer, pipeline::WGfxPipeline},
};

use super::viewport::Viewport;

#[repr(C)]
#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
pub struct RectVertex {
	#[format(R32G32_SINT)]
	pub in_pos: [i32; 2],
	#[format(R32_UINT)]
	pub in_dim: [u16; 2],
	#[format(R32_UINT)]
	pub in_color: u32,
	#[format(R32_UINT)]
	pub in_color2: u32,
	#[format(R32_UINT)]
	pub corner_radius_gradient_srgb: [u8; 4],
	#[format(R32_SFLOAT)]
	pub depth: f32,
}

/// Cloneable pipeline & shaders to be shared between RectRenderer instances.
#[derive(Clone)]
pub struct RectPipeline {
	gfx: Arc<WGfx>,
	pub(super) color_rect: Arc<WGfxPipeline<RectVertex>>,
}

impl RectPipeline {
	pub fn new(gfx: Arc<WGfx>, format: Format) -> anyhow::Result<Self> {
		let vert = vert_rect::load(gfx.device.clone())?;
		let frag = frag_rect::load(gfx.device.clone())?;

		let color_rect = gfx.create_pipeline::<RectVertex>(
			vert,
			frag,
			format,
			Some(BLEND_ALPHA),
			PrimitiveTopology::TriangleStrip,
			true,
		)?;

		Ok(Self { gfx, color_rect })
	}
}

pub struct RectRenderer {
	pipeline: RectPipeline,
	rect_vertices: Vec<RectVertex>,
	vert_buffer: Subbuffer<[RectVertex]>,
	vert_buffer_size: usize,
}

impl RectRenderer {
	pub fn new(pipeline: RectPipeline) -> anyhow::Result<Self> {
		const BUFFER_SIZE: usize = 128;

		let vert_buffer = pipeline.gfx.empty_buffer(
			BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
			BUFFER_SIZE as _,
		)?;

		Ok(Self {
			pipeline,
			rect_vertices: vec![],
			vert_buffer,
			vert_buffer_size: BUFFER_SIZE,
		})
	}

	pub fn add_rect(&mut self, boundary: Boundary, rectangle: Rectangle) {
		let clamped_radius = rectangle
			.round_radius
			.min(boundary.w / 2.0)
			.min(boundary.h / 2.0);
		let skew_radius = [clamped_radius / boundary.w, clamped_radius / boundary.h];

		self.rect_vertices.push(RectVertex {
			in_pos: [boundary.x as _, boundary.y as _],
			in_dim: [boundary.w as _, boundary.h as _],
			in_color: cosmic_text::Color::from(rectangle.color).0,
			in_color2: cosmic_text::Color::from(rectangle.color2).0,
			corner_radius_gradient_srgb: [
				skew_radius[0] as u8,
				skew_radius[1] as u8,
				rectangle.gradient as u8,
				0, //FIXME: srgb vs linear?
			],
			depth: 0.0, //FIXME: add depth
		});
	}

	fn upload_verts(&mut self) -> anyhow::Result<()> {
		if self.vert_buffer_size < self.rect_vertices.len() {
			let new_size = self.vert_buffer_size * 2;
			self.vert_buffer = self.pipeline.gfx.empty_buffer(
				BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
				new_size as _,
			)?;
			self.vert_buffer_size = new_size;
		}

		self.vert_buffer.write()?[0..self.rect_vertices.len()].clone_from_slice(&self.rect_vertices);

		Ok(())
	}

	pub fn render(
		&mut self,
		viewport: &mut Viewport,
		cmd_buf: &mut GfxCommandBuffer,
	) -> anyhow::Result<()> {
		let vp = viewport.resolution();

		let set0 = viewport.get_rect_descriptor(&self.pipeline);

		self.upload_verts()?;

		let pass = self.pipeline.color_rect.create_pass_instanced(
			[vp[0] as _, vp[1] as _],
			self.vert_buffer.clone(),
			0..4,
			0..self.rect_vertices.len() as _,
			vec![set0],
		)?;

		self.rect_vertices.clear();

		cmd_buf.run_ref(&pass)
	}
}

pub mod vert_rect {
	vulkano_shaders::shader! {
			ty: "vertex",
			src: r"#version 310 es
            precision highp float;

            layout (location = 0) in ivec2 in_pos;
            layout (location = 1) in uint in_dim;
            layout (location = 2) in uint in_color;
            layout (location = 3) in uint in_color2;
            layout (location = 4) in uint corner_radius_gradient_srgb;
            layout (location = 5) in float depth;

            layout (location = 0) out vec4 out_color;

            layout (set = 0, binding = 0) uniform UniformParams {
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

              uint corner_radius = corner_radius_gradient_srgb & 0xffu;

							uint gradient_mode = (corner_radius_gradient_srgb & 0x00ff0000u) >> 16;

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

              uint srgb = (corner_radius_gradient_srgb & 0xff000000u) >> 24;

              if (srgb == 0u) {
							  out_color = vec4(
									float((color & 0x00ff0000u) >> 16u) / 255.0,
									float((color & 0x0000ff00u) >> 8u) / 255.0,
									float(color & 0x000000ffu) / 255.0,
									float((color & 0xff000000u) >> 24u) / 255.0
								);
							} else {
							  out_color = vec4(
									srgb_to_linear(float((color & 0x00ff0000u) >> 16u) / 255.0),
									srgb_to_linear(float((color & 0x0000ff00u) >> 8u) / 255.0),
									srgb_to_linear(float(color & 0x000000ffu) / 255.0),
									float((color & 0xff000000u) >> 24u) / 255.0
								);
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

            layout (location = 0) out vec4 out_color;

            layout (set = 0, binding = 0) uniform UniformParams {
                uniform uvec2 screen_resolution;
            };

            void main()
            {
								out_color = in_color;
            }
        ",
	}
}
