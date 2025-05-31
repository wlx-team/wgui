use std::sync::Arc;

use cosmic_text::{Attrs, Buffer, Metrics, Shaping, Wrap};

use crate::{
	drawing::{self},
	renderer_vk::text::{FONT_SYSTEM, RenderableText, TextStyle},
};

use super::{WidgetObj, WidgetState};

#[derive(Default)]
pub struct TextParams {
	pub content: String,
	pub style: TextStyle,
}

pub struct TextLabel {
	params: TextParams,
	renderable: Option<Arc<RenderableText>>,
}

impl TextLabel {
	pub fn create(params: TextParams) -> anyhow::Result<WidgetState> {
		WidgetState::new(Box::new(Self {
			params,
			renderable: None,
		}))
	}

	pub fn set_text(&mut self, text: &str) {
		self.params.content = String::from(text);
		self.renderable = None; // invalidate text cache
	}
}

impl WidgetObj for TextLabel {
	fn draw(&mut self, state: &mut super::DrawState, _params: &super::DrawParams) {
		let boundary = drawing::Boundary::construct(state.transform_stack);
		let metrics = Metrics::from(&self.params.style);
		let attrs = Attrs::from(&self.params.style);
		let wrap = Wrap::from(&self.params.style);

		let mut buffer = Buffer::new_empty(metrics);

		{
			let mut font_system = FONT_SYSTEM.lock().unwrap(); // safe unwrap
			let mut buffer = buffer.borrow_with(&mut font_system);
			buffer.set_wrap(wrap);

			// set text last in order to avoid expensive re-shaping
			buffer.set_rich_text(
				[(self.params.content.as_str(), attrs)],
				&Attrs::new(),
				Shaping::Advanced,
				self.params.style.align.map(|a| a.into()),
			);
		}

		state
			.primitives
			.push(drawing::RenderPrimitive::Text(boundary, buffer));
	}

	fn measure(
		&mut self,
		_known_dimensions: taffy::Size<Option<f32>>,
		_available_space: taffy::Size<taffy::AvailableSpace>,
	) -> taffy::Size<f32> {
		let Some(_renderable) = &self.renderable else {
			return taffy::Size::ZERO;
		};

		// todo
		taffy::Size::ZERO
	}
}
