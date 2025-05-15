use tiny_skia::{Paint, Pixmap};
use wgui::{
	drawing::{self, RenderPrimitive},
	glam::Vec2,
	layout::Layout,
	taffy::{self, AlignContent, AlignItems, FlexDirection, prelude::length},
	widget::rectangle::{Rectangle, RectangleParams},
};

pub struct Testbed {
	layout: Layout,
}

impl Testbed {
	pub fn new() -> anyhow::Result<Self> {
		let mut layout = Layout::new()?;

		let container = layout.add_child(
			layout.root,
			Rectangle::new(RectangleParams {
				color: drawing::Color([0.8, 0.8, 0.8, 1.0]),
			})?,
			taffy::Style {
				flex_direction: FlexDirection::Row,
				size: taffy::Size::percent(1.0),
				..Default::default()
			},
		)?;

		let first_rect = layout.add_child(
			container,
			Rectangle::new(RectangleParams {
				color: drawing::Color([0.8, 0.5, 0.2, 1.0]),
			})?,
			taffy::Style {
				size: taffy::Size::percent(1.0),
				flex_wrap: taffy::FlexWrap::Wrap,
				align_items: Some(AlignItems::FlexStart),
				align_content: Some(AlignContent::FlexStart),
				..Default::default()
			},
		)?;

		let _second_rect = layout.add_child(
			container,
			Rectangle::new(RectangleParams {
				color: drawing::Color([0.5, 0.8, 0.2, 1.0]),
			})?,
			taffy::Style {
				size: taffy::Size::percent(1.0),
				..Default::default()
			},
		)?;

		let _third_rect = layout.add_child(
			container,
			Rectangle::new(RectangleParams {
				color: drawing::Color([0.2, 0.5, 0.8, 1.0]),
			})?,
			taffy::Style {
				size: taffy::Size::percent(1.0),
				..Default::default()
			},
		)?;

		// add some boxes to a first rect

		for i in 0..40 {
			let _ = layout.add_child(
				first_rect,
				Rectangle::new(RectangleParams {
					color: drawing::Color([1.0 - (i as f32 / 40.0), 0.0, i as f32 / 40.0, 1.0]),
				})?,
				taffy::Style {
					size: taffy::Size {
						width: length(4.0 + i as f32 * 2.0),
						height: length(32.0),
					},
					margin: taffy::Rect::length(8.0),
					..Default::default()
				},
			)?;
		}

		Ok(Self { layout })
	}

	pub fn update(&mut self, width: f32, height: f32) -> anyhow::Result<()> {
		self.layout.update(Vec2::new(width, height))?;
		Ok(())
	}

	pub fn draw(&self, pixmap: &mut Pixmap, transform: &tiny_skia::Transform) -> anyhow::Result<()> {
		let primitives = wgui::drawing::draw(&self.layout)?;
		for primitive in primitives {
			match primitive {
				RenderPrimitive::Rectangle(boundary, rectangle) => {
					let mut paint = Paint::default();
					let col = &rectangle.color.0;
					paint.set_color(tiny_skia::Color::from_rgba(col[0], col[1], col[2], col[3]).unwrap());

					pixmap.fill_rect(
						tiny_skia::Rect::from_xywh(boundary.x, boundary.y, boundary.w, boundary.h).unwrap(),
						&paint,
						*transform,
						None,
					);
				}
				RenderPrimitive::Image(_boundary, _image) => todo!(),
			}
		}

		Ok(())
	}
}
