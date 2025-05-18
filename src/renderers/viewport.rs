use std::sync::Arc;

use vulkano::{
	buffer::{BufferContents, BufferUsage, Subbuffer},
	descriptor_set::DescriptorSet,
};

use crate::gfx::WGfx;

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
	pub fn new(gfx: WGfx) -> anyhow::Result<Self> {
		let params = Params {
			screen_resolution: [0, 0],
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

	/// Updates the `Viewport` with the given `resolution`.
	pub fn update(&mut self, resolution: [u32; 2]) -> anyhow::Result<()> {
		if self.params.screen_resolution != resolution {
			self.params.screen_resolution = resolution;

			self.params_buffer.write()?.copy_from_slice(&[self.params]);
		}
		Ok(())
	}

	/// Returns the current resolution of the `Viewport`.
	pub fn resolution(&self) -> [u32; 2] {
		self.params.screen_resolution
	}
}

#[repr(C)]
#[derive(BufferContents, Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Params {
	pub screen_resolution: [u32; 2],
}
