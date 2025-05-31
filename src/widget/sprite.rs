use crate::{
	drawing::{self},
	renderer_vk::text::custom_glyph::{CustomGlyph, CustomGlyphId},
};

use super::{WidgetObj, WidgetState};

#[derive(Default)]
pub struct SpriteBoxParams {
	pub glyph_id: CustomGlyphId,
}

#[derive(Default)]
pub struct SpriteBox {
	params: SpriteBoxParams,
}

impl SpriteBox {
	pub fn create(params: SpriteBoxParams) -> anyhow::Result<WidgetState> {
		WidgetState::new(Box::new(Self { params }))
	}
}

impl WidgetObj for SpriteBox {
	fn draw(&mut self, state: &mut super::DrawState, _params: &super::DrawParams) {
		let boundary = drawing::Boundary::construct(state.transform_stack);

		let glyph = CustomGlyph {
			id: self.params.glyph_id,
			left: 0.0,
			top: 0.0,
			width: boundary.w,
			height: boundary.h,
			color: Some(cosmic_text::Color::rgb(255, 255, 255)),
			snap_to_physical_pixel: true,
			metadata: 0,
		};

		state
			.primitives
			.push(drawing::RenderPrimitive::Sprite(boundary, Some(glyph)));
	}

	fn measure(
		&mut self,
		_known_dimensions: taffy::Size<Option<f32>>,
		_available_space: taffy::Size<taffy::AvailableSpace>,
	) -> taffy::Size<f32> {
		//TODO: do we even need this?
		taffy::Size::ZERO
	}
}
