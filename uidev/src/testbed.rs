use tiny_skia::{Paint, Pixmap};
use wgui::{
	drawing::{self, RenderPrimitive},
	layout::Layout,
	widget::rectangle::{Rectangle, RectangleParams},
};

pub struct Testbed {
	layout: Layout,
}

impl Testbed {
	pub fn new() -> anyhow::Result<Self> {
		let mut layout = Layout::new()?;

		let rect = Box::new(Rectangle::new(
			layout.init_params(),
			RectangleParams {
				color: drawing::Color([0.8, 0.5, 0.2, 1.0]),
			},
		)?);

		layout.add_child(Some(layout.root), rect);

		Ok(Self { layout })
	}

	pub fn draw(&self, pixmap: &mut Pixmap, transform: &tiny_skia::Transform) {
		let primitives = wgui::drawing::draw(&self.layout);
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
	}
}
