mod custom_glyph;
mod shaders;
pub mod text_atlas;
pub mod text_renderer;

use std::sync::{LazyLock, Mutex};

use cosmic_text::{
	Align, Attrs, Buffer, Color, FontSystem, Metrics, Shaping, Style, SwashCache, Weight, Wrap,
};
use custom_glyph::{ContentType, CustomGlyph};
use etagere::AllocId;
use glam::ivec2;
use taffy::AvailableSpace;

use crate::drawing::{self, Boundary};

pub static FONT_SYSTEM: LazyLock<Mutex<FontSystem>> =
	LazyLock::new(|| Mutex::new(FontSystem::new()));
pub static SWASH_CACHE: LazyLock<Mutex<SwashCache>> =
	LazyLock::new(|| Mutex::new(SwashCache::new()));

/// Used in case no font_size is defined
const DEFAULT_FONT_SIZE: f32 = 14.;

/// In case no line_height is defined, use font_size * DEFAULT_LINE_HEIGHT_RATIO
const DEFAULT_LINE_HEIGHT_RATIO: f32 = 1.43;

pub struct RenderableText {
	buffer: Buffer,
}

impl RenderableText {
	/// Used to render text where the style is the same for the entire length.
	pub fn new(content: &str, style: &TextStyle) -> Self {
		let metrics = style.into();
		let attrs = style.into();
		let wrap = style.into();

		let mut buffer = Buffer::new_empty(metrics);

		{
			let mut font_system = FONT_SYSTEM.lock().unwrap(); // safe unwrap
			let mut buffer = buffer.borrow_with(&mut font_system);
			buffer.set_wrap(wrap);

			// set text last in order to avoid expensive re-shaping
			buffer.set_rich_text(
				[(content, attrs)],
				&Attrs::new(),
				Shaping::Advanced,
				style.align.map(|a| a.into()),
			);
		}

		Self { buffer }
	}

	pub fn get_buffer(&self) -> &Buffer {
		&self.buffer
	}

	pub fn measure(
		&mut self,
		known_dimensions: taffy::Size<Option<f32>>,
		available_space: taffy::Size<taffy::AvailableSpace>,
	) -> taffy::Size<f32> {
		// Set width constraint
		let width_constraint = known_dimensions.width.or(match available_space.width {
			AvailableSpace::MinContent => Some(0.0),
			AvailableSpace::MaxContent => None,
			AvailableSpace::Definite(width) => Some(width),
		});

		let mut font_system = FONT_SYSTEM.lock().unwrap(); // safe unwrap
		self
			.buffer
			.set_size(&mut font_system, width_constraint, None);

		// Compute layout
		self.buffer.shape_until_scroll(&mut font_system, false);

		// Determine measured size of text
		let (width, total_lines) = self
			.buffer
			.layout_runs()
			.fold((0.0, 0usize), |(width, total_lines), run| {
				(run.line_w.max(width), total_lines + 1)
			});
		let height = total_lines as f32 * self.buffer.metrics().line_height;

		taffy::Size { width, height }
	}

	pub fn draw<F>(&self, boundary: Boundary, mut f: F)
	where
		F: FnMut(i32, i32, u32, u32, drawing::Color),
	{
		const DEFAULT_COLOR: cosmic_text::Color = Color::rgb(0, 0, 0);

		let mut swash_cache = SWASH_CACHE.lock().unwrap(); // safe unwrap
		let mut font_system = FONT_SYSTEM.lock().unwrap(); // safe unwrap

		let pos = ivec2(boundary.x as _, boundary.y as _);

		self.buffer.draw(
			&mut font_system,
			&mut swash_cache,
			DEFAULT_COLOR, // color is set in via `attrs` in new()
			|x, y, w, h, color| f(x + pos.x, y + pos.y, w, h, color.into()),
		);
	}
}

#[derive(Default, Clone)]
pub struct TextStyle {
	pub size: Option<f32>,
	pub line_height: Option<f32>,
	pub color: Option<drawing::Color>, // TODO: should this be hex?
	pub style: Option<FontStyle>,
	pub weight: Option<FontWeight>,
	pub align: Option<HorizontalAlign>,
	pub wrap: bool,
}

impl From<&TextStyle> for Attrs<'_> {
	fn from(style: &TextStyle) -> Self {
		Attrs::new()
			.color(style.color.unwrap_or_default().into())
			.style(style.style.unwrap_or_default().into())
			.weight(style.weight.unwrap_or_default().into())
	}
}

impl From<&TextStyle> for Metrics {
	fn from(style: &TextStyle) -> Self {
		let font_size = style.size.unwrap_or(DEFAULT_FONT_SIZE);

		Metrics {
			font_size,
			line_height: style
				.size
				.unwrap_or_else(|| (font_size * DEFAULT_LINE_HEIGHT_RATIO).round()),
		}
	}
}

impl From<&TextStyle> for Wrap {
	fn from(value: &TextStyle) -> Self {
		if value.wrap {
			Wrap::WordOrGlyph
		} else {
			Wrap::None
		}
	}
}

// helper structs for serde

#[derive(Default, Debug, Clone, Copy)]
pub enum FontStyle {
	#[default]
	Normal,
	Italic,
}

impl From<FontStyle> for Style {
	fn from(value: FontStyle) -> Style {
		match value {
			FontStyle::Normal => Style::Normal,
			FontStyle::Italic => Style::Italic,
		}
	}
}

#[derive(Default, Debug, Clone, Copy)]
pub enum FontWeight {
	#[default]
	Normal,
	Bold,
}

impl From<FontWeight> for Weight {
	fn from(value: FontWeight) -> Weight {
		match value {
			FontWeight::Normal => Weight::NORMAL,
			FontWeight::Bold => Weight::BOLD,
		}
	}
}

#[derive(Default, Debug, Clone, Copy)]
pub enum HorizontalAlign {
	#[default]
	Left,
	Right,
	Center,
	Justified,
	End,
}

impl From<HorizontalAlign> for Align {
	fn from(value: HorizontalAlign) -> Align {
		match value {
			HorizontalAlign::Left => Align::Left,
			HorizontalAlign::Right => Align::Right,
			HorizontalAlign::Center => Align::Center,
			HorizontalAlign::Justified => Align::Justified,
			HorizontalAlign::End => Align::End,
		}
	}
}

impl From<drawing::Color> for cosmic_text::Color {
	fn from(value: drawing::Color) -> cosmic_text::Color {
		let [r, g, b, a] = value.0;
		cosmic_text::Color::rgba(
			(r * 255.999) as _,
			(g * 255.999) as _,
			(b * 255.999) as _,
			(a * 255.999) as _,
		)
	}
}

impl From<cosmic_text::Color> for drawing::Color {
	fn from(value: cosmic_text::Color) -> drawing::Color {
		drawing::Color([
			value.r() as f32 / 255.999,
			value.g() as f32 / 255.999,
			value.b() as f32 / 255.999,
			value.a() as f32 / 255.999,
		])
	}
}

// glyphon types below

pub(super) enum GpuCacheStatus {
	InAtlas {
		x: u16,
		y: u16,
		content_type: ContentType,
	},
	SkipRasterization,
}

pub(super) struct GlyphDetails {
	width: u16,
	height: u16,
	gpu_cache: GpuCacheStatus,
	atlas_id: Option<AllocId>,
	top: i16,
	left: i16,
}

/// Controls the visible area of the text. Any text outside of the visible area will be clipped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextBounds {
	/// The position of the left edge of the visible area.
	pub left: i32,
	/// The position of the top edge of the visible area.
	pub top: i32,
	/// The position of the right edge of the visible area.
	pub right: i32,
	/// The position of the bottom edge of the visible area.
	pub bottom: i32,
}

/// The default visible area doesn't clip any text.
impl Default for TextBounds {
	fn default() -> Self {
		Self {
			left: i32::MIN,
			top: i32::MIN,
			right: i32::MAX,
			bottom: i32::MAX,
		}
	}
}

/// A text area containing text to be rendered along with its overflow behavior.
#[derive(Clone)]
pub struct TextArea<'a> {
	/// The buffer containing the text to be rendered.
	pub buffer: &'a Buffer,
	/// The left edge of the buffer.
	pub left: f32,
	/// The top edge of the buffer.
	pub top: f32,
	/// The scaling to apply to the buffer.
	pub scale: f32,
	/// The visible bounds of the text area. This is used to clip the text and doesn't have to
	/// match the `left` and `top` values.
	pub bounds: TextBounds,
	/// The default color of the text area.
	pub default_color: Color,
	/// Additional custom glyphs to render.
	pub custom_glyphs: &'a [CustomGlyph],
}
