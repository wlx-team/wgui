use glam::Vec2;

#[derive(Default)]
pub struct Transform {
	pub pos: Vec2,

	pub dim: Vec2, // for convenience
}

impl Transform {
	pub fn pos(pos: Vec2) -> Transform {
		Transform {
			pos,
			dim: Default::default(),
		}
	}
}

const TRANSFORM_STACK_MAX: usize = 32;
pub struct TransformStack {
	pub stack: [Transform; TRANSFORM_STACK_MAX],
	top: u8,
}

impl TransformStack {
	pub fn new() -> Self {
		Self {
			stack: Default::default(),
			top: 1,
		}
	}

	pub fn push(&mut self, mut t: Transform) {
		assert!(self.top < TRANSFORM_STACK_MAX as u8);
		let idx = (self.top - 1) as usize;
		t.pos += self.stack[idx].pos;
		self.stack[self.top as usize] = t;
		self.top += 1;
	}

	pub fn pop(&mut self) {
		assert!(self.top > 0);
		self.top -= 1;
	}

	pub fn get(&self) -> &Transform {
		&self.stack[(self.top - 1) as usize]
	}

	pub fn get_pos(&self) -> Vec2 {
		self.stack[(self.top - 1) as usize].pos
	}
}

impl Default for TransformStack {
	fn default() -> Self {
		Self::new()
	}
}
