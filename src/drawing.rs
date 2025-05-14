use super::{
	layout::{BoxedWidget, Layout},
	widget::DrawParams,
};

pub struct ImageHandle {
	// to be implemented, will contain pixel data (RGB or RGBA) loaded via "ImageBank" or something by the gui
}

pub struct Boundary {
	pub x: f32,
	pub y: f32,
	pub w: f32,
	pub h: f32,
}

#[derive(Copy, Clone)]
pub struct Color(pub [f32; 4]);

impl Default for Color {
	fn default() -> Self {
		// opaque black
		Self([0.0, 0.0, 0.0, 1.0])
	}
}

#[derive(Default)]
pub struct Rectangle {
	pub color: Color,
	pub round_radius: f32, // 0.0 - 1.0
}

pub struct Image {
	_handle: ImageHandle,
}

pub enum RenderPrimitive {
	Rectangle(Boundary, Rectangle),
	Image(Boundary, Image),
}

fn draw_children(layout: &Layout, params: &mut DrawParams, widget: &BoxedWidget) {
	widget.draw(params);

	for handle in widget.data().children.iter() {
		let Some(child) = layout.widgets.get(handle) else {
			println!("warning: skipping invalid widget handle");
			continue;
		};

		draw_children(layout, params, child);
	}
}

pub fn draw(layout: &Layout) -> Vec<RenderPrimitive> {
	let Some(root) = layout.widgets.get(&layout.root) else {
		panic!("root widget doesn't exist"); // This shouldn't happen
	};

	let mut primitives = Vec::<RenderPrimitive>::new();

	let mut params = DrawParams {
		primitives: &mut primitives,
	};

	draw_children(layout, &mut params, root);

	primitives
}
