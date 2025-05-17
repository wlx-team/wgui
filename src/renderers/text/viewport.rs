use std::sync::Arc;

use vulkano::{
	buffer::{BufferContents, BufferUsage, Subbuffer},
	descriptor_set::DescriptorSet,
};

use super::text_atlas::TextResources;

/// Controls the visible area of all text for a given renderer. Any text outside of the visible
/// area will be clipped.
///
/// Many projects will only ever need a single `Viewport`, but it is possible to create multiple
/// `Viewport`s if you want to render text to specific areas within a window (without having to)
/// bound each `TextArea`).
pub struct Viewport {
	params: Params,
	params_buffer: Subbuffer<[Params]>,
	pub params_descriptor: Arc<DescriptorSet>,
}

impl Viewport {
	/// Creates a new `Viewport` with the given `device` and `cache`.
	pub fn new(common: TextResources) -> anyhow::Result<Self> {
		let params = Params {
			screen_resolution: [0, 0],
		};

		let params_buffer = common.gfx.new_buffer(
			BufferUsage::UNIFORM_BUFFER | BufferUsage::TRANSFER_DST,
			[params].iter(),
		)?;

		let params_descriptor = common.pipeline.uniform_buffer(2, params_buffer.clone())?;

		Ok(Self {
			params,
			params_buffer,
			params_descriptor,
		})
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
