use vulkano::buffer::Subbuffer;

use super::{Params, Resolution, cache::Cache};
use std::{mem, slice};

/// Controls the visible area of all text for a given renderer. Any text outside of the visible
/// area will be clipped.
///
/// Many projects will only ever need a single `Viewport`, but it is possible to create multiple
/// `Viewport`s if you want to render text to specific areas within a window (without having to)
/// bound each `TextArea`).
#[derive(Debug)]
pub struct Viewport {
	params: Params,
	params_buffer: Option<Subbuffer<u8>>,
}

impl Viewport {
	/// Creates a new `Viewport` with the given `device` and `cache`.
	pub fn new(cache: &Cache) -> Self {
		let params = Params {
			screen_resolution: Resolution {
				width: 0,
				height: 0,
			},
			_pad: [0, 0],
		};

		Self {
			params,
			params_buffer: None,
		}
	}

	/// Updates the `Viewport` with the given `resolution`.
	pub fn update(&mut self, queue: &Queue, resolution: Resolution) {
		if self.params.screen_resolution != resolution {
			self.params.screen_resolution = resolution;

			queue.write_buffer(&self.params_buffer, 0, unsafe {
				slice::from_raw_parts(
					&self.params as *const Params as *const u8,
					mem::size_of::<Params>(),
				)
			});
		}
	}

	/// Returns the current resolution of the `Viewport`.
	pub fn resolution(&self) -> Resolution {
		self.params.screen_resolution
	}
}
