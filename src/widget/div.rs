use taffy::Size;

use super::{WidgetObj, WidgetState};

pub struct Div {}

impl Div {
	pub fn create() -> anyhow::Result<WidgetState> {
		WidgetState::new(Box::new(Self {}))
	}
}

impl WidgetObj for Div {
	fn draw(&mut self, _params: &mut super::DrawParams) {
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
