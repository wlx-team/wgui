use crate::drawing;

use super::{InitParams, Widget, WidgetData};

pub struct RectangleParams {
	pub color: drawing::Color,
}

pub struct Rectangle {
	data: WidgetData,
	params: RectangleParams,
}

impl Rectangle {
	pub fn new(mut init_params: InitParams, params: RectangleParams) -> anyhow::Result<Self> {
		Ok(Self {
			data: WidgetData::from_params(&mut init_params)?,
			params,
		})
	}
}

impl Widget for Rectangle {
	fn data_mut(&mut self) -> &mut WidgetData {
		&mut self.data
	}

	fn data(&self) -> &WidgetData {
		&self.data
	}

	fn draw(&self, params: &mut super::DrawParams) {
		params.primitives.push(drawing::RenderPrimitive::Rectangle(
			self.data.boundary(),
			drawing::Rectangle {
				color: self.params.color,
				round_radius: 0.0,
			},
		));
	}
}
