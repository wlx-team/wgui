use glam::Vec2;

use crate::{
	layout::BoxWidget,
	renderers::text::RenderableText,
	transform_stack::{Transform, TransformStack},
};

use super::{layout::Layout, widget::DrawParams};

pub struct ImageHandle {
	// to be implemented, will contain pixel data (RGB or RGBA) loaded via "ImageBank" or something by the gui
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Boundary {
	pub x: f32,
	pub y: f32,
	pub w: f32,
	pub h: f32,
}

impl Boundary {
	pub fn construct(transform_stack: &TransformStack) -> Self {
		let transform = transform_stack.get();

		Self {
			x: transform.pos.x,
			y: transform.pos.y,
			w: transform.dim.x,
			h: transform.dim.y,
		}
	}
}

#[derive(Copy, Clone)]
pub struct Color(pub [f32; 4]);

impl Default for Color {
	fn default() -> Self {
		// opaque black
		Self([0.0, 0.0, 0.0, 1.0])
	}
}

#[repr(u8)]
#[derive(Default, Clone, Copy)]
pub enum GradientMode {
	#[default]
	None,
	Horizontal,
	Vertical,
}

#[derive(Default, Clone, Copy)]
pub struct Rectangle {
	pub color: Color,
	pub color2: Color,
	pub gradient: GradientMode,

	pub round_radius: f32, // 0.0 - 1.0
}

pub struct Image {
	_handle: ImageHandle,
}

pub enum RenderPrimitive {
	Rectangle(Boundary, Rectangle),
	Text(Boundary, RenderableText),
	Image(Boundary, Image),
}

fn draw_children(layout: &Layout, params: &mut DrawParams, widget: &BoxWidget) {
	let Ok(l) = layout.tree.layout(widget.data().node) else {
		debug_assert!(false);
		return;
	};

	params.transform_stack.push(Transform {
		pos: Vec2::new(l.location.x, l.location.y),
		dim: Vec2::new(l.size.width, l.size.height),
	});

	widget.draw(params);

	for child_id in widget.data().children.iter() {
		let Some(child) = layout.widgets.get(*child_id) else {
			println!("warning: skipping invalid widget id");
			continue;
		};

		params.current_widget = *child_id;
		draw_children(layout, params, child);
	}

	params.transform_stack.pop();
}

pub fn draw(layout: &Layout) -> anyhow::Result<Vec<RenderPrimitive>> {
	let Some(root) = layout.widgets.get(layout.root) else {
		panic!("root widget doesn't exist"); // This shouldn't happen
	};

	let mut primitives = Vec::<RenderPrimitive>::new();
	let mut transform_stack = TransformStack::new();

	let mut params = DrawParams {
		current_widget: layout.root,
		primitives: &mut primitives,
		transform_stack: &mut transform_stack,
		layout,
	};

	draw_children(layout, &mut params, root);

	Ok(primitives)
}
