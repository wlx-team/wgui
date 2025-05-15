use tiny_skia::{Paint, Pixmap};
use wgui::{
	drawing::{self, RenderPrimitive},
	layout::Layout,
	taffy::{self, prelude::length},
	widget::rectangle::{Rectangle, RectangleParams},
};

pub struct Testbed {
	layout: Layout,
}

impl Testbed {
	pub fn new() -> anyhow::Result<Self> {
		let mut layout = Layout::new()?;

		let rect = layout.add_child(
			layout.root,
			Rectangle::new(RectangleParams {
				color: drawing::Color([0.8, 0.5, 0.2, 1.0]),
			})?,
			taffy::Style {
				size: taffy::Size {
					width: length(128.0),
					height: length(32.0),
				},
				margin: taffy::Rect {
					top: length(8.0),
					left: length(8.0),
					right: length(8.0),
					bottom: length(8.0),
				},
				..Default::default()
			},
		)?;

		let subrect = layout.add_child(
			rect,
			Rectangle::new(RectangleParams {
				color: drawing::Color([1.0, 0.2, 0.6, 1.0]),
			})?,
			taffy::Style {
				size: taffy::Size {
					width: length(64.0),
					height: length(64.0),
				},
				margin: taffy::Rect {
					top: length(8.0),
					left: length(8.0),
					right: length(8.0),
					bottom: length(8.0),
				},
				..Default::default()
			},
		)?;

		let _subsubrect = layout.add_child(
			subrect,
			Rectangle::new(RectangleParams {
				color: drawing::Color([0.0, 0.2, 1.0, 1.0]),
			})?,
			taffy::Style {
				size: taffy::Size {
					width: length(96.0),
					height: length(32.0),
				},
				margin: taffy::Rect {
					top: length(8.0),
					left: length(8.0),
					right: length(8.0),
					bottom: length(8.0),
				},
				..Default::default()
			},
		)?;

		Ok(Self { layout })
	}

	pub fn update(&mut self, width: f32, height: f32) -> anyhow::Result<()> {
		let root_node = self
			.layout
			.widgets
			.get(&self.layout.root)
			.unwrap()
			.data()
			.node;

		if self.layout.tree.dirty(root_node)? {
			println!("re-computing layout");
			self.layout.tree.compute_layout(
				root_node,
				taffy::Size {
					width: taffy::AvailableSpace::Definite(width),
					height: taffy::AvailableSpace::Definite(height),
				},
			)?;
		}

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
