use std::sync::{Arc, Mutex};

use taffy::Size;

use crate::drawing::{self, GradientMode};

use super::{Widget, WidgetState};

#[derive(Default)]
pub struct RectangleParams {
	pub color: drawing::Color,
	pub color2: drawing::Color,
	pub gradient: GradientMode,
	pub round: f32,
}

pub struct Rectangle {
	data: WidgetState,
	params: RectangleParams,
}

impl Rectangle {
	pub fn new(params: RectangleParams) -> anyhow::Result<Arc<Mutex<Self>>> {
		Ok(Arc::new(Mutex::new(Self {
			data: WidgetState::new()?,
			params,
		})))
	}
}

impl Widget for Rectangle {
	fn state_mut(&mut self) -> &mut WidgetState {
		&mut self.data
	}

	fn state(&self) -> &WidgetState {
		&self.data
	}

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
