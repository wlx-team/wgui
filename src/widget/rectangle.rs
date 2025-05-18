use taffy::Size;

use crate::drawing::{self, GradientMode};

use super::{WidgetObj, WidgetState};

#[derive(Default)]
pub struct RectangleParams {
	pub color: drawing::Color,
	pub color2: drawing::Color,
	pub gradient: GradientMode,
	pub round: f32,
}

pub struct Rectangle {
	pub params: RectangleParams,
}

impl Rectangle {
	pub fn create(params: RectangleParams) -> anyhow::Result<WidgetState> {
		WidgetState::new(Box::new(Rectangle { params }))
	}
}

impl WidgetObj for Rectangle {
	fn draw(&self, params: &mut super::DrawParams) {
		params.primitives.push(drawing::RenderPrimitive::Rectangle(
			drawing::Boundary::construct(params.transform_stack),
			drawing::Rectangle {
				color: self.params.color,
				color2: self.params.color2,
				gradient: self.params.gradient,
				round: self.params.round,
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
