use tiny_skia::{Paint, Pixmap};
use wgui::{
	drawing::{self, RenderPrimitive},
	glam::Vec2,
	layout::Layout,
};

pub struct Testbed {
	pub layout: Layout,
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

		// create some buttons for testing
		for i in 0..10 {
			let n = i as f32 / 10.0;
			button::construct(
				&mut layout,
				my_div_parent,
				button::Params {
					text: "I'm a button!",
					color: drawing::Color([1.0 - n, n * n, n, 1.0]),
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
				RenderPrimitive::Text(boundary, text) => {
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
