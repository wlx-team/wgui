use std::{
	ops::Deref,
	sync::{Arc, Mutex},
};

use vulkano::{
	buffer::{BufferContents, Subbuffer},
	format::{self, Format},
	pipeline::{
		GraphicsPipeline, PipelineLayout,
		graphics::{
			depth_stencil::DepthStencilState, multisample::MultisampleState, vertex_input::Vertex,
		},
	},
};

use crate::wgfx::{BLEND_ALPHA, WGfx};

use super::shaders::{frag_atlas, vert_atlas};

/// A cache to share common resources (e.g., pipelines, layouts, shaders) between multiple text
/// renderers.
#[derive(Clone)]
pub struct Cache {
	pub gfx: Arc<WGfx>,

	inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
	sampler: Sampler,
	vertex_buffers: Subbuffer<u8>,
	atlas_layout: BindGroupLayout,
	uniforms_layout: BindGroupLayout,
	pipeline_layout: PipelineLayout,
	cache: Mutex<
		Vec<(
			Format,
			MultisampleState,
			Option<DepthStencilState>,
			Arc<GraphicsPipeline>,
		)>,
	>,
}

impl Cache {
	pub fn new(wgfx: Arc<WGfx>) -> anyhow::Result<Self> {
		let vert = vert_atlas::load(wgfx.device.clone())?;
		let frag = frag_atlas::load(wgfx.device.clone())?;

		let pipeline = wgfx.create_pipeline::<AtlasVertIn>(vert, frag, format, Some(BLEND_ALPHA))?;

		Self(Arc::new(Inner {
			sampler,
			shader,
			vertex_buffers: [vertex_buffer_layout],
			uniforms_layout,
			atlas_layout,
			pipeline_layout,
			cache: Mutex::new(Vec::new()),
		}))
	}

	pub(crate) fn get_or_create_pipeline(
		&self,
		format: Format,
		multisample: MultisampleState,
		depth_stencil: Option<DepthStencilState>,
	) -> Arc<GraphicsPipeline> {
		let Inner {
			cache,
			pipeline_layout,
			shader,
			vertex_buffers,
			..
		} = self.inner.deref();

		let mut cache = cache.lock().expect("Write pipeline cache");

		cache
			.iter()
			.find(|(fmt, ms, ds, _)| fmt == &format && ms == &multisample && ds == &depth_stencil)
			.map(|(_, _, _, p)| p.clone())
			.unwrap_or_else(|| {
				cache.push((format, multisample, depth_stencil, pipeline.clone()));

				pipeline
			})
			.clone()
	}
}

#[repr(C)]
#[derive(BufferContents, Vertex, Copy, Clone, Debug)]
pub struct AtlasVertIn {
	#[format(R32G32_SINT)]
	pub in_pos: [i32; 2],
	#[format(R16G16_UINT)]
	pub dim: [u16; 2],
	#[format(R16G16_UINT)]
	pub uv: [u16; 2],
	#[format(R8G8B8A8_UINT)]
	pub color: [u8; 4],
	#[format(R16G16_UINT)]
	pub content_type_with_srgb: [u16; 2],
	#[format(R32_SFLOAT)]
	pub depth: f32,
}
