use std::sync::Arc;

use glam::Vec3;
use vulkano::{
	buffer::{BufferContents, BufferUsage, Subbuffer},
	format::Format,
	pipeline::graphics::{input_assembly::PrimitiveTopology, vertex_input::Vertex},
};

use crate::{
	drawing::{Boundary, Rectangle},
	gfx::{BLEND_ALPHA, WGfx, cmd::GfxCommandBuffer, pipeline::WGfxPipeline},
	renderer_vk::model_buffer::ModelBuffer,
};

use super::viewport::Viewport;

#[repr(C)]
#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
pub struct RectVertex {
	#[format(R32_UINT)]
	pub in_model_idx: u32,
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
	model_buffer: ModelBuffer,
	rot: f32,
}

impl RectRenderer {
	pub fn new(pipeline: RectPipeline, rot: f32) -> anyhow::Result<Self> {
		const BUFFER_SIZE: usize = 128;

		let vert_buffer = pipeline.gfx.empty_buffer(
			BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
			BUFFER_SIZE as _,
		)?;

		Ok(Self {
			model_buffer: ModelBuffer::new(&pipeline.gfx)?,
			pipeline,
			rect_vertices: vec![],
			vert_buffer,
			vert_buffer_size: BUFFER_SIZE,
			rot,
		})
	}

	pub fn add_rect(
		&mut self,
		viewport: &Viewport,
		boundary: Boundary,
		rectangle: Rectangle,
		scale: f32,
		depth: f32,
	) {
		// TODO: use projection matrix instead of this abomination with positions and dimensions
		let res = viewport.resolution();
		let shift = Vec3::new(
			(boundary.x + boundary.w / 2.0) / res[0] as f32,
			(boundary.y + boundary.h / 2.0) / res[1] as f32,
			0.0,
		);
		let vec_scale = Vec3::new(res[0] as f32 / res[1] as f32, 1.0, 1.0); // aspect
		let vec_scale_inv = Vec3::new(res[1] as f32 / res[0] as f32, 1.0, 1.0); // inverse aspect

		let mut model = glam::Mat4::IDENTITY;

		model *= glam::Mat4::from_scale(vec_scale_inv);
		model *= glam::Mat4::from_translation(-shift);
		model *= glam::Mat4::from_rotation_z(self.rot + boundary.y);
		model *= glam::Mat4::from_translation(shift);
		model *= glam::Mat4::from_scale(vec_scale);

		let in_model_idx = self.model_buffer.register(&model);

		self.rect_vertices.push(RectVertex {
			in_model_idx,
			in_pos: [(boundary.x * scale) as _, (boundary.y * scale) as _],
			in_dim: [(boundary.w * scale) as _, (boundary.h * scale) as _],
			in_color: cosmic_text::Color::from(rectangle.color).0,
			in_color2: cosmic_text::Color::from(rectangle.color2).0,
			in_border_color: cosmic_text::Color::from(rectangle.border_color).0,
			round_border_gradient_srgb: [
				(rectangle.round * scale * 255.0) as u8,
				(rectangle.border * scale) as u8,
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
		gfx: &Arc<WGfx>,
		viewport: &mut Viewport,
		cmd_buf: &mut GfxCommandBuffer,
	) -> anyhow::Result<()> {
		let vp = viewport.resolution();

		let set0 = viewport.get_rect_descriptor(&self.pipeline);
		let set1 = self.model_buffer.get_rect_descriptor(&self.pipeline);

		self.model_buffer.upload(gfx)?;
		self.upload_verts()?;

		let pass = self.pipeline.color_rect.create_pass_instanced(
			[vp[0] as _, vp[1] as _],
			self.vert_buffer.clone(),
			0..4,
			0..self.rect_vertices.len() as _,
			vec![set0, set1],
		)?;

		self.rect_vertices.clear();

		cmd_buf.run_ref(&pass)
	}
}

pub mod vert_rect {
	vulkano_shaders::shader! {
			ty: "vertex",
			path: "src/renderer_vk/shaders/rect.vert",
	}
}

pub mod frag_rect {
	vulkano_shaders::shader! {
			ty: "fragment",
			path: "src/renderer_vk/shaders/rect.frag",
	}
}
