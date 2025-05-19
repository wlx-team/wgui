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
	pub in_border_color: u32,
	#[format(R32_UINT)]
	pub round_border_gradient_srgb: [u8; 4],
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

	pub fn add_rect(&mut self, boundary: Boundary, rectangle: Rectangle, depth: f32) {
		self.rect_vertices.push(RectVertex {
			in_pos: [boundary.x as _, boundary.y as _],
			in_dim: [boundary.w as _, boundary.h as _],
			in_color: cosmic_text::Color::from(rectangle.color).0,
			in_color2: cosmic_text::Color::from(rectangle.color2).0,
			in_border_color: cosmic_text::Color::from(rectangle.border_color).0,
			round_border_gradient_srgb: [
				(rectangle.round * 255.0) as u8,
				rectangle.border as u8,
				rectangle.gradient as u8,
				0, //FIXME: srgb vs linear?
			],
			depth,
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
			path: "src/renderers/rect.vert",
	}
}

pub mod frag_rect {
	vulkano_shaders::shader! {
			ty: "fragment",
			path: "src/renderers/rect.frag",
	}
}
