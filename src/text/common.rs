use std::sync::Arc;

use vulkano::{buffer::BufferContents, format::Format, pipeline::graphics::vertex_input::Vertex};

use super::shaders::{frag_atlas, vert_atlas};
use crate::wgfx::{BLEND_ALPHA, WGfx, WGfxPipeline};

/// A cache to share common resources between multiple text renderers.
#[derive(Clone)]
pub struct CommonResources {
	pub gfx: Arc<WGfx>,
	pub pipeline: Arc<WGfxPipeline<GlyphVertex>>,
}

impl CommonResources {
	pub fn new(gfx: Arc<WGfx>, format: Format) -> anyhow::Result<Self> {
		let vert = vert_atlas::load(gfx.device.clone())?;
		let frag = frag_atlas::load(gfx.device.clone())?;

		let pipeline = gfx.create_pipeline::<GlyphVertex>(vert, frag, format, Some(BLEND_ALPHA))?;

		Ok(Self { gfx, pipeline })
	}
}

#[repr(C)]
#[derive(BufferContents, Vertex, Copy, Clone, Debug, Default)]
pub struct GlyphVertex {
	#[format(R32G32_SINT)]
	pub in_pos: [i32; 2],
	#[format(R16G16_UINT)]
	pub dim: [u16; 2],
	#[format(R16G16_UINT)]
	pub uv: [u16; 2],
	#[format(R32_UINT)]
	pub color: u32,
	#[format(R16G16_UINT)]
	pub content_type_with_srgb: [u16; 2],
	#[format(R32_SFLOAT)]
	pub depth: f32,
}