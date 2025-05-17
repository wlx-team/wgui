use taffy::Size;

use super::{Widget, WidgetData};

pub struct Div {
	data: WidgetData,
}

impl Div {
	pub fn new() -> anyhow::Result<Box<Self>> {
		Ok(Box::new(Self {
			data: WidgetData::new()?,
		}))
	}
}

impl Widget for Div {
	fn data_mut(&mut self) -> &mut WidgetData {
		&mut self.data
	}

	fn data(&self) -> &WidgetData {
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
