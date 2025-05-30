use std::sync::Arc;

use vulkano::{
	buffer::{BufferContents, BufferUsage, Subbuffer},
	descriptor_set::DescriptorSet,
};

use crate::{gfx::WGfx, renderer_vk::model_buffer};

use super::{rect::RectPipeline, text::text_atlas::TextPipeline};

/// Controls the visible area of all text for a given renderer. Any text outside of the visible
/// area will be clipped.
pub struct Viewport {
	params: Params,
	params_buffer: Subbuffer<[Params]>,
	text_descriptor: Option<Arc<DescriptorSet>>,
	rect_descriptor: Option<Arc<DescriptorSet>>,
}

impl Viewport {
	/// Creates a new `Viewport` with the given `device` and `cache`.
	pub fn new(gfx: Arc<WGfx>) -> anyhow::Result<Self> {
		let params = Params {
			screen_resolution: [0, 0],
			padding1: [0, 0],
			models: [0.0; MODELS_F32_COUNT],
		};

		let params_buffer = gfx.new_buffer(
			BufferUsage::UNIFORM_BUFFER | BufferUsage::TRANSFER_DST,
			[params].iter(),
		)?;

		Ok(Self {
			params,
			params_buffer,
			text_descriptor: None,
			rect_descriptor: None,
		})
	}

	pub fn get_text_descriptor(&mut self, pipeline: &TextPipeline) -> Arc<DescriptorSet> {
		self
			.text_descriptor
			.get_or_insert_with(|| {
				pipeline
					.inner
					.uniform_buffer(2, self.params_buffer.clone())
					.unwrap() // safe unwrap
			})
			.clone()
	}

	pub fn get_rect_descriptor(&mut self, pipeline: &RectPipeline) -> Arc<DescriptorSet> {
		self
			.rect_descriptor
			.get_or_insert_with(|| {
				pipeline
					.color_rect
					.uniform_buffer(0, self.params_buffer.clone())
					.unwrap() // safe unwrap
			})
			.clone()
	}

	pub fn set_resolution(&mut self, resolution: [u32; 2]) {
		self.params.screen_resolution = resolution;
	}

	pub fn set_model_buffer(&mut self, buf: &model_buffer::ModelBuffer) {
		unsafe {
			std::ptr::copy_nonoverlapping::<f32>(
				buf.models.as_slice().as_ptr() as *const f32,
				self.params.models.as_mut_ptr(),
				MODELS_F32_COUNT,
			);
		}
	}

	pub fn update(&mut self) -> anyhow::Result<()> {
		self.params_buffer.write()?.copy_from_slice(&[self.params]);
		Ok(())
	}

	/// Returns the current resolution of the `Viewport`.
	pub fn resolution(&self) -> [u32; 2] {
		self.params.screen_resolution
	}
}

const MODELS_F32_COUNT: usize = model_buffer::MAX_COUNT * (4 * 4);

#[repr(C)]
#[derive(BufferContents, Clone, Copy, Debug, PartialEq)]
pub(crate) struct Params {
	// 0-3, 4 bytes
	pub screen_resolution: [u32; 2],
	pub padding1: [u32; 2], // data alignment (vec4 size)

	// 4-2063, 2048 bytes
	pub models: [f32; MODELS_F32_COUNT], // 2048 bytes, mat4 ModelBuffer array
}
