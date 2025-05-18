use glam::Vec2;
use taffy::TraversePartialTree;

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
pub struct Color {
	pub r: f32,
	pub g: f32,
	pub b: f32,
	pub a: f32,
}

impl Color {
	pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
		Self { r, g, b, a }
	}
}

impl Default for Color {
	fn default() -> Self {
		// opaque black
		Self::new(0.0, 0.0, 0.0, 1.0)
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

	pub round: f32, // 0.0 - 1.0
}

pub struct Image {
	_handle: ImageHandle,
}

pub enum RenderPrimitive {
	Rectangle(Boundary, Rectangle),
	Text(Boundary, RenderableText),
	Image(Boundary, Image),
}

fn draw_widget(
	layout: &Layout,
	params: &mut DrawParams,
	node_id: taffy::NodeId,
	widget: &BoxWidget,
) {
	let Ok(l) = layout.tree.layout(node_id) else {
		debug_assert!(false);
		return;
	};

	params.transform_stack.push(Transform {
		pos: Vec2::new(l.location.x, l.location.y),
		dim: Vec2::new(l.size.width, l.size.height),
	});

	widget.lock().unwrap().obj.draw(params);

	draw_children(layout, params, node_id);

	params.transform_stack.pop();
}

fn draw_children(layout: &Layout, params: &mut DrawParams, parent_node_id: taffy::NodeId) {
	for node_id in layout.tree.child_ids(parent_node_id) {
		let Some(widget_id) = layout.tree.get_node_context(node_id).cloned() else {
			debug_assert!(false);
			continue;
		};

		let Some(widget) = layout.widget_states.get(widget_id) else {
			debug_assert!(false);
			continue;
		};

		draw_widget(layout, params, node_id, widget);
	}
}

pub fn draw(layout: &Layout) -> anyhow::Result<Vec<RenderPrimitive>> {
	let mut primitives = Vec::<RenderPrimitive>::new();
	let mut transform_stack = TransformStack::new();

	let mut params = DrawParams {
		primitives: &mut primitives,
		transform_stack: &mut transform_stack,
		layout,
	};

	let Some(root_widget) = layout.widget_states.get(layout.root_widget) else {
		panic!("root widget doesn't exist"); // This shouldn't happen
	};

	draw_widget(layout, &mut params, layout.root_node, root_widget);

	Ok(primitives)
}
