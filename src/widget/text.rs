use crate::{
	drawing::{self},
	renderers::text::{RenderableText, TextStyle},
};

use super::{WidgetObj, WidgetState};

#[derive(Default)]
pub struct TextParams {
	pub content: String,
	pub style: TextStyle,
}

pub struct TextLabel {
	params: TextParams,
}

impl TextLabel {
	pub fn create(params: TextParams) -> anyhow::Result<WidgetState> {
		WidgetState::new(Box::new(Self { params }))
	}
}

impl WidgetObj for TextLabel {
	fn draw(&self, params: &mut super::DrawParams) {
		let boundary = drawing::Boundary::construct(params.transform_stack);
		let renderable = RenderableText::new(&self.params.content, &self.params.style);

		params
			.primitives
			.push(drawing::RenderPrimitive::Text(boundary, renderable));
	}

	fn measure(
		&mut self,
		_known_dimensions: taffy::Size<Option<f32>>,
		_available_space: taffy::Size<taffy::AvailableSpace>,
	) -> taffy::Size<f32> {
		todo!();
	}
}
