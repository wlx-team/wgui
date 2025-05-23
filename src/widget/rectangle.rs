use crate::drawing;

use super::{Widget, WidgetData};

pub struct RectangleParams {
	pub color: drawing::Color,
}

pub struct Rectangle {
	data: WidgetData,
	params: RectangleParams,
}

impl Rectangle {
	pub fn new(params: RectangleParams) -> anyhow::Result<Box<Self>> {
		Ok(Box::new(Self {
			data: WidgetData::new()?,
			params,
		}))
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
			drawing::Boundary::construct(params.transform_stack),
			drawing::Rectangle {
				color: self.params.color,
				round_radius: 0.0,
			},
		));
	}
}
