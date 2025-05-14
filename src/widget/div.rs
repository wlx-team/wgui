use super::{InitParams, Widget, WidgetData};

pub struct Div {
	data: WidgetData,
}

impl Div {
	pub fn new(params: &mut InitParams) -> anyhow::Result<Self> {
		Ok(Self {
			data: WidgetData::from_params(params)?,
		})
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
}
