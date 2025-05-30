pub const MAX_COUNT: usize = 64; // see uniform.glsl

pub struct ModelBuffer {
	pub models: [glam::Mat4; MAX_COUNT],
	pub idx: u32,
}

impl ModelBuffer {
	pub fn new() -> Self {
		Self {
			models: [glam::Mat4::IDENTITY; MAX_COUNT],
			idx: 0,
		}
	}

	// Returns model matrix ID from the model
	// TODO: faster lookup instead of this quick-n-dirty thing.
	// Should work just fine if there are only a few transformations,
	// 99% of them will be IDENTITY anyways with index 0
	pub fn register(&mut self, model: &glam::Mat4) -> u32 {
		for (idx, iter_model) in self.models.iter().enumerate() {
			if iter_model == model {
				return idx as u32;
			}
		}

		if self.idx == MAX_COUNT as u32 {
			log::error!("ModelBuffer is full, returning 0");
			return 0;
		}

		// insert new
		self.models[self.idx as usize] = *model;
		let ret = self.idx;
		self.idx += 1;
		ret
	}
}
