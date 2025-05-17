use crate::{
	drawing::{self},
	renderers::text::{RenderableText, TextStyle},
};

use super::{Widget, WidgetData};

#[derive(Default)]
pub struct TextParams {
	pub content: String,
	pub style: TextStyle,
}

pub struct TextLabel {
	data: WidgetData,
	params: TextParams,
}

impl TextLabel {
	pub fn new(params: TextParams) -> anyhow::Result<Box<Self>> {
		Ok(Box::new(Self {
			data: WidgetData::new()?,
			params,
		}))
	}
}

impl Widget for TextLabel {
	fn data_mut(&mut self) -> &mut WidgetData {
		&mut self.data
	}

	fn data(&self) -> &WidgetData {
		&self.data
	}

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
