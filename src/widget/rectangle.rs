use crate::drawing::{self, GradientMode};

use super::{WidgetObj, WidgetState};

#[derive(Default)]
pub struct RectangleParams {
	pub color: drawing::Color,
	pub color2: drawing::Color,
	pub gradient: GradientMode,

	pub border: f32,
	pub border_color: drawing::Color,

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
	fn draw(&mut self, state: &mut super::DrawState, _params: &super::DrawParams) {
		state.primitives.push(drawing::RenderPrimitive {
			boundary: drawing::Boundary::construct(state.transform_stack),
			depth: state.depth,
			payload: drawing::PrimitivePayload::Rectangle(drawing::Rectangle {
				color: self.params.color,
				color2: self.params.color2,
				gradient: self.params.gradient,
				border: self.params.border,
				border_color: self.params.border_color,
				round: self.params.round,
			}),
		});
	}
}
