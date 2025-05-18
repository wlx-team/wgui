use std::sync::{Arc, Mutex};

use taffy::Size;

use super::{Widget, WidgetState};

pub struct Div {
	data: WidgetState,
}

impl Div {
	pub fn new() -> anyhow::Result<Arc<Mutex<Self>>> {
		Ok(Arc::new(Mutex::new(Self {
			data: WidgetState::new()?,
		})))
	}
}

impl Widget for Div {
	fn state_mut(&mut self) -> &mut WidgetState {
		&mut self.data
	}

	fn state(&self) -> &WidgetState {
		&self.data
	}

	fn draw(&self, _params: &mut super::DrawParams) {
		// no-op
	}

	fn measure(
		&mut self,
		_: taffy::Size<Option<f32>>,
		_: taffy::Size<taffy::AvailableSpace>,
	) -> taffy::Size<f32> {
		Size::ZERO
	}
}
