use crate::{gfx::cmd::GfxCommandBuffer, renderers::viewport::Viewport};

use super::{
	ContentType, FontSystem, GlyphDetails, GpuCacheStatus, SwashCache, TextArea,
	custom_glyph::{CustomGlyphCacheKey, RasterizeCustomGlyphRequest, RasterizedCustomGlyph},
	text_atlas::{ColorMode, GlyphVertex, TextAtlas, TextPipeline},
};
use cosmic_text::{Color, SubpixelBin, SwashContent};
use vulkano::{
	buffer::{BufferUsage, Subbuffer},
	command_buffer::CommandBufferUsage,
};

/// A text renderer that uses cached glyphs to render text into an existing render pass.
pub struct TextRenderer {
	pipeline: TextPipeline,
	vertex_buffer: Subbuffer<[GlyphVertex]>,
	vertex_buffer_capacity: usize,
	glyph_vertices: Vec<GlyphVertex>,
}

impl TextRenderer {
	/// Creates a new `TextRenderer`.
	pub fn new(atlas: &mut TextAtlas) -> anyhow::Result<Self> {
		// A buffer element is a single quad with a glyph on it
		const INITIAL_CAPACITY: usize = 256;

		let vertex_buffer = atlas.common.gfx.empty_buffer(
			BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
			INITIAL_CAPACITY as _,
		)?;

		Ok(Self {
			pipeline: atlas.common.clone(),
			vertex_buffer,
			vertex_buffer_capacity: INITIAL_CAPACITY,
			glyph_vertices: Vec::new(),
		})
	}

	/// Prepares all of the provided text areas for rendering.
	pub fn prepare<'a>(
		&mut self,
		font_system: &mut FontSystem,
		atlas: &mut TextAtlas,
		viewport: &Viewport,
		text_areas: impl IntoIterator<Item = TextArea<'a>>,
		cache: &mut SwashCache,
	) -> anyhow::Result<()> {
		self.prepare_with_depth_and_custom(font_system, atlas, viewport, text_areas, cache, |_| None)
	}

	/// Prepares all of the provided text areas for rendering.
	pub fn prepare_with_custom<'a>(
		&mut self,
		font_system: &mut FontSystem,
		atlas: &mut TextAtlas,
		viewport: &Viewport,
		text_areas: impl IntoIterator<Item = TextArea<'a>>,
		cache: &mut SwashCache,
		rasterize_custom_glyph: impl FnMut(RasterizeCustomGlyphRequest) -> Option<RasterizedCustomGlyph>,
	) -> anyhow::Result<()> {
		self.prepare_with_depth_and_custom(
			font_system,
			atlas,
			viewport,
			text_areas,
			cache,
			rasterize_custom_glyph,
		)
	}

	/// Prepares all of the provided text areas for rendering.
	#[allow(clippy::too_many_arguments)]
	pub fn prepare_with_depth_and_custom<'a>(
		&mut self,
		font_system: &mut FontSystem,
		atlas: &mut TextAtlas,
		viewport: &Viewport,
		text_areas: impl IntoIterator<Item = TextArea<'a>>,
		cache: &mut SwashCache,
		mut rasterize_custom_glyph: impl FnMut(RasterizeCustomGlyphRequest) -> Option<RasterizedCustomGlyph>,
	) -> anyhow::Result<()> {
		self.glyph_vertices.clear();

		let resolution = viewport.resolution();

		for text_area in text_areas {
			let bounds_min_x = text_area.bounds.left.max(0);
			let bounds_min_y = text_area.bounds.top.max(0);
			let bounds_max_x = text_area.bounds.right.min(resolution[0] as i32);
			let bounds_max_y = text_area.bounds.bottom.min(resolution[1] as i32);

			for glyph in text_area.custom_glyphs.iter() {
				let x = text_area.left + (glyph.left * text_area.scale);
				let y = text_area.top + (glyph.top * text_area.scale);
				let width = (glyph.width * text_area.scale).round() as u16;
				let height = (glyph.height * text_area.scale).round() as u16;

				let (x, y, x_bin, y_bin) = if glyph.snap_to_physical_pixel {
					(
						x.round() as i32,
						y.round() as i32,
						SubpixelBin::Zero,
						SubpixelBin::Zero,
					)
				} else {
					let (x, x_bin) = SubpixelBin::new(x);
					let (y, y_bin) = SubpixelBin::new(y);
					(x, y, x_bin, y_bin)
				};

				let cache_key = GlyphonCacheKey::Custom(CustomGlyphCacheKey {
					glyph_id: glyph.id,
					width,
					height,
					x_bin,
					y_bin,
				});

				let color = glyph.color.unwrap_or(text_area.default_color);

				if let Some(glyph_to_render) = prepare_glyph(
					x,
					y,
					0.0,
					color,
					cache_key,
					atlas,
					cache,
					font_system,
					text_area.scale,
					bounds_min_x,
					bounds_min_y,
					bounds_max_x,
					bounds_max_y,
					text_area.depth,
					|_cache, _font_system, rasterize_custom_glyph| -> Option<GetGlyphImageResult> {
						if width == 0 || height == 0 {
							return None;
						}

						let input = RasterizeCustomGlyphRequest {
							id: glyph.id,
							width,
							height,
							x_bin,
							y_bin,
							scale: text_area.scale,
						};

						let output = (rasterize_custom_glyph)(input)?;

						output.validate(&input, None);

						Some(GetGlyphImageResult {
							content_type: output.content_type,
							top: 0,
							left: 0,
							width,
							height,
							data: output.data,
						})
					},
					&mut rasterize_custom_glyph,
				)? {
					self.glyph_vertices.push(glyph_to_render);
				}
			}

			let is_run_visible = |run: &cosmic_text::LayoutRun| {
				let start_y_physical = (text_area.top + (run.line_top * text_area.scale)) as i32;
				let end_y_physical = start_y_physical + (run.line_height * text_area.scale) as i32;

				start_y_physical <= text_area.bounds.bottom && text_area.bounds.top <= end_y_physical
			};

			let layout_runs = text_area
				.buffer
				.layout_runs()
				.skip_while(|run| !is_run_visible(run))
				.take_while(is_run_visible);

			for run in layout_runs {
				for glyph in run.glyphs.iter() {
					let physical_glyph = glyph.physical((text_area.left, text_area.top), text_area.scale);

					let color = match glyph.color_opt {
						Some(some) => some,
						None => text_area.default_color,
					};

					if let Some(glyph_to_render) = prepare_glyph(
						physical_glyph.x,
						physical_glyph.y,
						run.line_y,
						color,
						GlyphonCacheKey::Text(physical_glyph.cache_key),
						atlas,
						cache,
						font_system,
						text_area.scale,
						bounds_min_x,
						bounds_min_y,
						bounds_max_x,
						bounds_max_y,
						text_area.depth,
						|cache, font_system, _rasterize_custom_glyph| -> Option<GetGlyphImageResult> {
							let image = cache.get_image_uncached(font_system, physical_glyph.cache_key)?;

							let content_type = match image.content {
								SwashContent::Color => ContentType::Color,
								SwashContent::Mask => ContentType::Mask,
								SwashContent::SubpixelMask => {
									// Not implemented yet, but don't panic if this happens.
									ContentType::Mask
								}
							};

							Some(GetGlyphImageResult {
								content_type,
								top: image.placement.top as i16,
								left: image.placement.left as i16,
								width: image.placement.width as u16,
								height: image.placement.height as u16,
								data: image.data,
							})
						},
						&mut rasterize_custom_glyph,
					)? {
						self.glyph_vertices.push(glyph_to_render);
					}
				}
			}
		}

		let will_render = !self.glyph_vertices.is_empty();
		if !will_render {
			return Ok(());
		}

		let vertices = self.glyph_vertices.as_slice();

		while self.vertex_buffer_capacity < vertices.len() {
			let new_capacity = self.vertex_buffer_capacity * 2;
			self.vertex_buffer = self.pipeline.gfx.empty_buffer(
				BufferUsage::VERTEX_BUFFER | BufferUsage::TRANSFER_DST,
				new_capacity as _,
			)?;
			self.vertex_buffer_capacity = new_capacity;
		}
		self.vertex_buffer.write()?[..vertices.len()].clone_from_slice(vertices);

		Ok(())
	}

	/// Renders all layouts that were previously provided to `prepare`.
	pub fn render(
		&self,
		atlas: &TextAtlas,
		viewport: &mut Viewport,
		cmd_buf: &mut GfxCommandBuffer,
	) -> anyhow::Result<()> {
		if self.glyph_vertices.is_empty() {
			return Ok(());
		}

		let descriptor_sets = vec![
			atlas.color_atlas.image_descriptor.clone(),
			atlas.mask_atlas.image_descriptor.clone(),
			viewport.get_text_descriptor(&self.pipeline),
		];

		let res = viewport.resolution();

		let pass = self.pipeline.inner.create_pass_instanced(
			[res[0] as _, res[1] as _],
			self.vertex_buffer.clone(),
			0..4,
			0..self.glyph_vertices.len() as u32,
			descriptor_sets,
		)?;

		cmd_buf.run_ref(&pass)
	}
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum TextColorConversion {
	None = 0,
	ConvertToLinear = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum GlyphonCacheKey {
	Text(cosmic_text::CacheKey),
	Custom(CustomGlyphCacheKey),
}

struct GetGlyphImageResult {
	content_type: ContentType,
	top: i16,
	left: i16,
	width: u16,
	height: u16,
	data: Vec<u8>,
}

#[allow(clippy::too_many_arguments)]
fn prepare_glyph<R>(
	x: i32,
	y: i32,
	line_y: f32,
	color: Color,
	cache_key: GlyphonCacheKey,
	atlas: &mut TextAtlas,
	cache: &mut SwashCache,
	font_system: &mut FontSystem,
	scale_factor: f32,
	bounds_min_x: i32,
	bounds_min_y: i32,
	bounds_max_x: i32,
	bounds_max_y: i32,
	depth: f32,
	get_glyph_image: impl FnOnce(&mut SwashCache, &mut FontSystem, &mut R) -> Option<GetGlyphImageResult>,
	mut rasterize_custom_glyph: R,
) -> anyhow::Result<Option<GlyphVertex>>
where
	R: FnMut(RasterizeCustomGlyphRequest) -> Option<RasterizedCustomGlyph>,
{
	let gfx = atlas.common.gfx.clone();
	let details = if let Some(details) = atlas.mask_atlas.glyph_cache.get(&cache_key) {
		atlas.mask_atlas.glyphs_in_use.insert(cache_key);
		details
	} else if let Some(details) = atlas.color_atlas.glyph_cache.get(&cache_key) {
		atlas.color_atlas.glyphs_in_use.insert(cache_key);
		details
	} else {
		let Some(image) = (get_glyph_image)(cache, font_system, &mut rasterize_custom_glyph) else {
			return Ok(None);
		};

		let should_rasterize = image.width > 0 && image.height > 0;

		let (gpu_cache, atlas_id, inner) = if should_rasterize {
			let mut inner = atlas.inner_for_content_mut(image.content_type);

			// Find a position in the packer
			let allocation = loop {
				match inner.try_allocate(image.width as usize, image.height as usize) {
					Some(a) => break a,
					None => {
						if !atlas.grow(
							font_system,
							cache,
							image.content_type,
							scale_factor,
							&mut rasterize_custom_glyph,
						)? {
							anyhow::bail!(
								"Atlas full. atlas: {:?} cache_key: {:?}",
								image.content_type,
								cache_key
							);
						}

						inner = atlas.inner_for_content_mut(image.content_type);
					}
				}
			};
			let atlas_min = allocation.rectangle.min;

			let mut cmd_buf = gfx.create_xfer_command_buffer(CommandBufferUsage::OneTimeSubmit)?;

			cmd_buf.update_image(
				inner.image_view.image().clone(),
				&image.data,
				[atlas_min.x as _, atlas_min.y as _, 0],
				Some([image.width as _, image.height as _, 1]),
			)?;

			cmd_buf.build_and_execute_now()?; //TODO: do not wait for fence here

			(
				GpuCacheStatus::InAtlas {
					x: atlas_min.x as u16,
					y: atlas_min.y as u16,
					content_type: image.content_type,
				},
				Some(allocation.id),
				inner,
			)
		} else {
			let inner = &mut atlas.color_atlas;
			(GpuCacheStatus::SkipRasterization, None, inner)
		};

		inner.glyphs_in_use.insert(cache_key);
		// Insert the glyph into the cache and return the details reference
		inner.glyph_cache.get_or_insert(cache_key, || GlyphDetails {
			width: image.width,
			height: image.height,
			gpu_cache,
			atlas_id,
			top: image.top,
			left: image.left,
		})
	};

	let mut x = x + details.left as i32;
	let mut y = (line_y * scale_factor).round() as i32 + y - details.top as i32;

	let (mut atlas_x, mut atlas_y, content_type) = match details.gpu_cache {
		GpuCacheStatus::InAtlas { x, y, content_type } => (x, y, content_type),
		GpuCacheStatus::SkipRasterization => return Ok(None),
	};

	let mut width = details.width as i32;
	let mut height = details.height as i32;

	// Starts beyond right edge or ends beyond left edge
	let max_x = x + width;
	if x > bounds_max_x || max_x < bounds_min_x {
		return Ok(None);
	}

	// Starts beyond bottom edge or ends beyond top edge
	let max_y = y + height;
	if y > bounds_max_y || max_y < bounds_min_y {
		return Ok(None);
	}

	// Clip left ege
	if x < bounds_min_x {
		let right_shift = bounds_min_x - x;

		x = bounds_min_x;
		width = max_x - bounds_min_x;
		atlas_x += right_shift as u16;
	}

	// Clip right edge
	if x + width > bounds_max_x {
		width = bounds_max_x - x;
	}

	// Clip top edge
	if y < bounds_min_y {
		let bottom_shift = bounds_min_y - y;

		y = bounds_min_y;
		height = max_y - bounds_min_y;
		atlas_y += bottom_shift as u16;
	}

	// Clip bottom edge
	if y + height > bounds_max_y {
		height = bounds_max_y - y;
	}

	Ok(Some(GlyphVertex {
		in_pos: [x, y],
		in_dim: [width as u16, height as u16],
		in_uv: [atlas_x, atlas_y],
		in_color: color.0,
		content_type_with_srgb: [
			content_type as u16,
			match atlas.color_mode {
				ColorMode::Accurate => TextColorConversion::ConvertToLinear,
				ColorMode::Web => TextColorConversion::None,
			} as u16,
		],
		depth,
	}))
}
