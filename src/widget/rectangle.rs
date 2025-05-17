use taffy::Size;

use crate::drawing::{self, GradientMode};

use super::{Widget, WidgetData};

#[derive(Default)]
pub struct RectangleParams {
	pub color: drawing::Color,
	pub color2: drawing::Color,
	pub gradient: GradientMode,
	pub radius: f32,
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
		let mut color = self.params.color;

		// Just for demonstration purposes (test events). FIXME: Remove later!
		if self.data.hovered {
			color.0[0] *= 0.8;
			color.0[1] *= 0.8;
			color.0[2] *= 0.8;
		}

		params.primitives.push(drawing::RenderPrimitive::Rectangle(
			drawing::Boundary::construct(params.transform_stack),
			drawing::Rectangle {
				color,
				color2: self.params.color2,
				gradient: self.params.gradient,
				round_radius: self.params.radius,
			},
		));
	}

	fn measure(
		&mut self,
		_: taffy::Size<Option<f32>>,
		_: taffy::Size<taffy::AvailableSpace>,
	) -> taffy::Size<f32> {
		Size::ZERO
	}
}
