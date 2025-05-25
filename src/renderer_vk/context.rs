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

struct RendererPass<'a> {
	submitted: bool,
	text_areas: Vec<TextArea<'a>>,
	text_renderer: TextRenderer,
	rect_renderer: RectRenderer,
}

impl RendererPass<'_> {
	fn new(text_atlas: &mut TextAtlas, rect_pipeline: RectPipeline) -> anyhow::Result<Self> {
		let text_renderer = TextRenderer::new(text_atlas)?;
		let rect_renderer = RectRenderer::new(rect_pipeline)?;

		Ok(Self {
			submitted: false,
			text_renderer,
			rect_renderer,
			text_areas: Vec::new(),
		})
	}

	fn submit(
		&mut self,
		viewport: &mut Viewport,
		cmd_buf: &mut GfxCommandBuffer,
		text_atlas: &mut TextAtlas,
	) -> anyhow::Result<()> {
		if self.submitted {
			return Ok(());
		}
		self.submitted = true;

		self.rect_renderer.render(viewport, cmd_buf)?;

		{
			let mut font_system = FONT_SYSTEM.lock().unwrap();
			let mut swash_cache = SWASH_CACHE.lock().unwrap();

			self.text_renderer.prepare(
				&mut font_system,
				text_atlas,
				viewport,
				std::mem::take(&mut self.text_areas),
				&mut swash_cache,
			)?;
		}

		self.text_renderer.render(text_atlas, viewport, cmd_buf)?;

		Ok(())
	}
}

pub struct Context {
	viewport: Viewport,
	text_atlas: TextAtlas,
	rect_pipeline: RectPipeline,
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
		let text_atlas = TextAtlas::new(text_pipeline.clone())?;

		Ok(Self {
			viewport,
			text_atlas,
			rect_pipeline,
			text_pipeline,
			scale,
		})
	}

	pub fn regen(&mut self) -> anyhow::Result<()> {
		self.text_atlas = TextAtlas::new(self.text_pipeline.clone())?;
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

	fn new_pass(&mut self, passes: &mut Vec<RendererPass>) -> anyhow::Result<()> {
		passes.push(RendererPass::new(
			&mut self.text_atlas,
			self.rect_pipeline.clone(),
		)?);

		Ok(())
	}

	fn submit_pass(
		&mut self,
		cmd_buf: &mut GfxCommandBuffer,
		pass: &mut RendererPass,
	) -> anyhow::Result<()> {
		pass.submit(&mut self.viewport, cmd_buf, &mut self.text_atlas)?;
		Ok(())
	}

	pub fn draw(
		&mut self,
		cmd_buf: &mut GfxCommandBuffer,
		primitives: &[drawing::RenderPrimitive],
	) -> anyhow::Result<()> {
		let mut passes = Vec::<RendererPass>::new();
		self.new_pass(&mut passes)?;

		for primitive in primitives.iter() {
			let pass = passes.last_mut().unwrap(); // always safe

			match primitive {
				drawing::RenderPrimitive::Submit => {
					self.submit_pass(cmd_buf, pass)?;
					self.new_pass(&mut passes)?;
				}
				drawing::RenderPrimitive::Rectangle(boundary, rectangle) => {
					pass
						.rect_renderer
						.add_rect(*boundary, *rectangle, self.scale, 0.0);
				}
				drawing::RenderPrimitive::Text(boundary, text) => {
					pass.text_areas.push(TextArea {
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

		let pass = passes.last_mut().unwrap();
		self.submit_pass(cmd_buf, pass)?;

		Ok(())
	}
}
