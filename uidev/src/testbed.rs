use tiny_skia::{Paint, Pixmap};
use wgui::{
	drawing::{self, Color, RenderPrimitive},
	glam::Vec2,
	layout::Layout,
	taffy::{self, AlignContent, AlignItems, FlexDirection, prelude::length},
	text::{FontWeight, HorizontalAlign, TextStyle},
	widget::{
		rectangle::{Rectangle, RectangleParams},
		text::{TextLabel, TextParams},
	},
};

pub struct Testbed {
	layout: Layout,
}

impl Testbed {
	pub fn new() -> anyhow::Result<Self> {
		let mut layout = Layout::new()?;

		use wgui::components::button;
		let parent = layout.root;

		let res = wgui::parser::parse(
			&mut layout,
			parent,
			std::fs::read_to_string("res/testbed.xml").unwrap().as_str(),
		)?;

		let my_div_parent = res.require_by_id("my_div_parent")?;

		button::construct(
			&mut layout,
			my_div_parent,
			button::Params {
				text: "I'm a button!",
			},
		)?;

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
				RenderPrimitive::Text(boundary, mut text) => {
					text.draw(boundary, |x, y, w, h, color| {
						let mut paint = Paint::default();
						let col = color.0;
						paint.set_color(tiny_skia::Color::from_rgba(col[0], col[1], col[2], col[3]).unwrap());

						pixmap.fill_rect(
							tiny_skia::Rect::from_xywh(x as _, y as _, w as _, h as _).unwrap(),
							&paint,
							*transform,
							None,
						);
					});
				}
				RenderPrimitive::Image(_boundary, _image) => todo!(),
			}
		}

		Ok(())
	}
}
