use std::sync::Arc;

use glam::Vec2;
use taffy::TraversePartialTree;

use crate::{
	layout::BoxWidget,
	renderers::text::RenderableText,
	transform_stack::{self, Transform, TransformStack},
	widget,
};

use super::{layout::Layout, widget::DrawState};

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
	pub fn from_pos_size(pos: &Vec2, size: &Vec2) -> Self {
		Self {
			x: pos.x,
			y: pos.y,
			w: size.x,
			h: size.y,
		}
	}

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
	Radial,
}

#[derive(Default, Clone, Copy)]
pub struct Rectangle {
	pub color: Color,
	pub color2: Color,
	pub gradient: GradientMode,

	pub border: f32, // width in pixels
	pub border_color: Color,

	pub round: f32, // 0.0 - 1.0
}

pub struct Image {
	_handle: ImageHandle,
}

pub enum RenderPrimitive {
	Rectangle(Boundary, Rectangle),
	Text(Boundary, Arc<RenderableText>),
	Image(Boundary, Image),
}

fn draw_widget(
	layout: &Layout,
	state: &mut DrawState,
	node_id: taffy::NodeId,
	style: &taffy::Style,
	widget: &BoxWidget,
) {
	let Ok(l) = layout.tree.layout(node_id) else {
		debug_assert!(false);
		return;
	};

	let mut widget_state = widget.lock().unwrap();

	let (shift, info) = match widget::get_scrollbar_info(l) {
		Some(info) => (widget_state.get_scroll_shift(&info, l), Some(info)),
		None => (Vec2::default(), None),
	};

	state.transform_stack.push(transform_stack::Transform {
		pos: Vec2::new(l.location.x, l.location.y) - shift,
		dim: Vec2::new(l.size.width, l.size.height),
	});

	let draw_params = widget::DrawParams {
		node_id,
		taffy_layout: l,
		style,
	};

	widget_state.draw_all(state, &draw_params);

	draw_children(layout, state, node_id);

	state.transform_stack.pop();

	if let Some(info) = &info {
		widget_state.draw_scrollbars(state, &draw_params, info);
	}
}

fn draw_children(layout: &Layout, state: &mut DrawState, parent_node_id: taffy::NodeId) {
	for node_id in layout.tree.child_ids(parent_node_id) {
		let Some(widget_id) = layout.tree.get_node_context(node_id).cloned() else {
			debug_assert!(false);
			continue;
		};

		let Ok(style) = layout.tree.style(node_id) else {
			debug_assert!(false);
			continue;
		};

		let Some(widget) = layout.widget_states.get(widget_id) else {
			debug_assert!(false);
			continue;
		};

		draw_widget(layout, state, node_id, style, widget);
	}
}

pub fn draw(layout: &Layout) -> anyhow::Result<Vec<RenderPrimitive>> {
	let mut primitives = Vec::<RenderPrimitive>::new();
	let mut transform_stack = TransformStack::new();

	let Some(root_widget) = layout.widget_states.get(layout.root_widget) else {
		panic!();
	};

	let Ok(style) = layout.tree.style(layout.root_node) else {
		panic!();
	};

	let mut params = DrawState {
		primitives: &mut primitives,
		transform_stack: &mut transform_stack,
		layout,
	};

	draw_widget(layout, &mut params, layout.root_node, style, root_widget);

	Ok(primitives)
}
