use std::sync::Arc;

use crate::{
	drawing,
	gfx::{WGfx, cmd::GfxCommandBuffer},
};

use super::{
	rect::{RectPipeline, RectRenderer},
	text::{
		FONT_SYSTEM, SWASH_CACHE, TextArea, TextBounds,
		text_atlas::{TextAtlas, TextPipeline},
		text_renderer::TextRenderer,
	},
	viewport::Viewport,
};

pub struct Context {
	viewport: Viewport,
	text_renderer: TextRenderer,
	text_atlas: TextAtlas,
	rect_renderer: RectRenderer,

	text_pipeline: TextPipeline,
	scale: f32,
}

impl Context {
	pub fn new(
		gfx: Arc<WGfx>,
		native_format: vulkano::format::Format,
		scale: f32,
	) -> anyhow::Result<Self> {
		let rect_pipeline = RectPipeline::new(gfx.clone(), native_format)?;
		let text_pipeline = TextPipeline::new(gfx.clone(), native_format)?;

		let viewport = Viewport::new(gfx.clone())?;
		let mut text_atlas = TextAtlas::new(text_pipeline.clone())?;
		let text_renderer = TextRenderer::new(&mut text_atlas)?;
		let rect_renderer = RectRenderer::new(rect_pipeline)?;

		Ok(Self {
			viewport,
			text_renderer,
			rect_renderer,
			text_atlas,
			text_pipeline,
			scale,
		})
	}

	pub fn regen(&mut self) -> anyhow::Result<()> {
		self.text_atlas = TextAtlas::new(self.text_pipeline.clone())?;
		self.text_renderer = TextRenderer::new(&mut self.text_atlas)?;
		Ok(())
	}

	pub fn update_viewport(&mut self, resolution: [u32; 2], scale: f32) -> anyhow::Result<()> {
		if self.scale != scale {
			self.scale = scale;
			self.regen()?;
		}
		self.viewport.update(resolution)?;
		Ok(())
	}

	pub fn draw(
		&mut self,
		cmd_buf: &mut GfxCommandBuffer,
		primitives: &[drawing::RenderPrimitive],
	) -> anyhow::Result<()> {
		let mut text_areas = vec![];

		for primitive in primitives.iter() {
			match primitive {
				drawing::RenderPrimitive::Rectangle(boundary, rectangle) => {
					self
						.rect_renderer
						.add_rect(*boundary, *rectangle, self.scale, 0.0);
				}
				drawing::RenderPrimitive::Text(boundary, text) => {
					text_areas.push(TextArea {
						buffer: text.get_buffer(),
						left: boundary.x * self.scale,
						top: boundary.y * self.scale,
						bounds: TextBounds::default(), //FIXME: just using boundary coords here doesn't work
						scale: self.scale,
						default_color: cosmic_text::Color::rgb(255, 0, 0),
						custom_glyphs: &[],
						depth: 0.0, //FIXME: add depth info
					});
				}
				drawing::RenderPrimitive::Image(_boundary, _image) => todo!(),
			}
		}

		self.rect_renderer.render(&mut self.viewport, cmd_buf)?;

		{
			let mut font_system = FONT_SYSTEM.lock().unwrap();
			let mut swash_cache = SWASH_CACHE.lock().unwrap();

			self.text_renderer.prepare(
				&mut font_system,
				&mut self.text_atlas,
				&self.viewport,
				text_areas,
				&mut swash_cache,
			)?;
		}

		self
			.text_renderer
			.render(&self.text_atlas, &mut self.viewport, cmd_buf)?;

		Ok(())
	}
}
