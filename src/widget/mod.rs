use glam::Vec2;

use super::drawing::RenderPrimitive;
use crate::{
	any::AnyTrait,
	drawing,
	event::{CallbackData, Event, EventListener, MouseWheelEvent},
	layout::{Layout, WidgetID, WidgetMap},
	transform_stack::{self, TransformStack},
};

pub mod div;
pub mod rectangle;
pub mod text;

pub struct WidgetState {
	pub hovered: bool,
	pub pressed: bool,
	pub event_listeners: Vec<EventListener>,
	pub scrolling: Vec2, // normalized, 0.0-1.0. Not used in case if overflow != scroll
	pub obj: Box<dyn WidgetObj>,
}

impl WidgetState {
	fn new(obj: Box<dyn WidgetObj>) -> anyhow::Result<WidgetState> {
		Ok(Self {
			hovered: false,
			pressed: false,
			event_listeners: Vec::new(),
			scrolling: Vec2::default(),
			obj,
		})
	}
}

// global draw params
pub struct DrawState<'a> {
	pub layout: &'a Layout,
	pub primitives: &'a mut Vec<RenderPrimitive>,
	pub transform_stack: &'a mut TransformStack,
}

// per-widget draw params
pub struct DrawParams<'a> {
	pub node_id: taffy::NodeId,
	pub style: &'a taffy::Style,
	pub taffy_layout: &'a taffy::Layout,
}

pub trait WidgetObj: AnyTrait {
	fn draw(&mut self, state: &mut DrawState, params: &DrawParams);
	fn measure(
		&mut self,
		known_dimensions: taffy::Size<Option<f32>>,
		available_space: taffy::Size<taffy::AvailableSpace>,
	) -> taffy::Size<f32>;
}

pub struct EventParams<'a> {
	pub node_id: taffy::NodeId,
	pub style: &'a taffy::Style,
	pub taffy_layout: &'a taffy::Layout,
	pub widgets: &'a WidgetMap,
	pub tree: &'a taffy::TaffyTree<WidgetID>,
	pub transform_stack: &'a TransformStack,
}

pub enum EventResult {
	Pass,
	Consumed,
	Outside,
}

fn get_scroll_enabled(style: &taffy::Style) -> (bool, bool) {
	(
		style.overflow.x == taffy::Overflow::Scroll,
		style.overflow.y == taffy::Overflow::Scroll,
	)
}

pub struct ScrollbarInfo {
	// total contents size of the currently scrolling widget
	content_size: Vec2,
	// 0.0 - 1.0
	// 1.0: scrollbar handle not visible (inactive)
	handle_size: Vec2,
	overflow: Vec2,
}

pub fn get_scrollbar_info(l: &taffy::Layout) -> Option<ScrollbarInfo> {
	let overflow = Vec2::new(l.scroll_width(), l.scroll_height());
	if overflow.x == 0.0 && overflow.y == 0.0 {
		return None; // not overflowing
	}

	let content_size = Vec2::new(l.content_size.width, l.content_size.height);
	let handle_size = 1.0 - (overflow / content_size);

	Some(ScrollbarInfo {
		content_size,
		handle_size,
		overflow,
	})
}

impl dyn WidgetObj {
	pub fn get_as<T: 'static>(&self) -> &T {
		let any = self.as_any();
		any.downcast_ref::<T>().unwrap()
	}

	pub fn get_as_mut<T: 'static>(&mut self) -> &mut T {
		let any = self.as_any_mut();
		any.downcast_mut::<T>().unwrap()
	}
}

impl WidgetState {
	pub fn add_event_listener(&mut self, listener: EventListener) {
		self.event_listeners.push(listener);
	}

	pub fn get_scroll_shift(&self, info: &ScrollbarInfo, l: &taffy::Layout) -> Vec2 {
		Vec2::new(
			(info.content_size.x - l.content_box_width()) * self.scrolling.x,
			(info.content_size.y - l.content_box_height()) * self.scrolling.y,
		)
	}

	pub fn draw_all(&mut self, state: &mut DrawState, params: &DrawParams) {
		self.obj.draw(state, params);
	}

	pub fn draw_scrollbars(
		&mut self,
		state: &mut DrawState,
		params: &DrawParams,
		info: &ScrollbarInfo,
	) {
		let (enabled_horiz, enabled_vert) = get_scroll_enabled(params.style);
		if !enabled_horiz && !enabled_vert {
			return;
		}

		let transform = state.transform_stack.get();

		let thickness = 6.0;
		let margin = 4.0;

		let rect_params = drawing::Rectangle {
			color: drawing::Color::new(1.0, 1.0, 1.0, 0.0),
			border: 2.0,
			border_color: drawing::Color::new(1.0, 1.0, 1.0, 1.0),
			round: 1.0,
			..Default::default()
		};

		// Horizontal handle
		if enabled_horiz && info.handle_size.x < 1.0 {
			state.primitives.push(drawing::RenderPrimitive::Rectangle(
				drawing::Boundary::from_pos_size(
					&Vec2::new(
						transform.pos.x + transform.dim.x * (1.0 - info.handle_size.x) * self.scrolling.x,
						transform.pos.y + transform.dim.y - thickness - margin,
					),
					&Vec2::new(transform.dim.x * info.handle_size.x, thickness),
				),
				rect_params,
			));
		}

		// Vertical handle
		if enabled_vert && info.handle_size.y < 1.0 {
			state.primitives.push(drawing::RenderPrimitive::Rectangle(
				drawing::Boundary::from_pos_size(
					&Vec2::new(
						transform.pos.x + transform.dim.x - thickness - margin,
						transform.pos.y + transform.dim.y * (1.0 - info.handle_size.y) * self.scrolling.y,
					),
					&Vec2::new(thickness, transform.dim.y * info.handle_size.y),
				),
				rect_params,
			));
		}
	}

	fn process_wheel(&mut self, params: &mut EventParams, wheel: &MouseWheelEvent) -> bool {
		let (enabled_horiz, enabled_vert) = get_scroll_enabled(params.style);
		if !enabled_horiz && !enabled_vert {
			return false;
		}

		let l = params.taffy_layout;
		let overflow = Vec2::new(l.scroll_width(), l.scroll_height());
		if overflow.x == 0.0 && overflow.y == 0.0 {
			return false; // not overflowing
		}

		let Some(info) = get_scrollbar_info(params.taffy_layout) else {
			return false;
		};

		let step_pixels = 32.0;

		if info.handle_size.x < 1.0 && wheel.pos.x != 0.0 {
			// Horizontal scrolling
			let mult = (1.0 / (l.content_box_width() - info.content_size.x)) * step_pixels;
			self.scrolling.x = (self.scrolling.x + wheel.shift.x * mult).clamp(0.0, 1.0);
		}

		if info.handle_size.y < 1.0 && wheel.pos.y != 0.0 {
			// Vertical scrolling
			let mult = (1.0 / (l.content_box_height() - info.content_size.y)) * step_pixels;
			self.scrolling.y = (self.scrolling.y + wheel.shift.y * mult).clamp(0.0, 1.0);
		}

		true
	}

	pub fn process_event(
		&mut self,
		widget_id: WidgetID,
		node_id: taffy::NodeId,
		event: &Event,
		params: &mut EventParams,
	) -> EventResult {
		let hovered = event.test_mouse_within_transform(params.transform_stack.get());

		let mut just_clicked = false;
		match &event {
			Event::MouseDown(_) => {
				if self.hovered {
					self.pressed = true;
				}
			}
			Event::MouseUp(_) => {
				if self.pressed {
					self.pressed = false;
					just_clicked = self.hovered;
				}
			}
			Event::MouseWheel(e) => {
				if self.process_wheel(params, e) {
					return EventResult::Consumed;
				}
			}
			_ => {}
		}

		for listener in &self.event_listeners {
			let mut data = CallbackData {
				obj: self.obj.as_mut(),
				widgets: params.widgets,
				widget_id,
				node_id,
			};

			match listener {
				EventListener::MouseEnter(callback) => {
					if hovered && !self.hovered {
						callback(&mut data);
					}
				}
				EventListener::MouseLeave(callback) => {
					if !hovered && !self.hovered {
						callback(&mut data);
					}
				}
				EventListener::MouseClick(callback) => {
					if just_clicked {
						callback(&mut data);
					}
				}
			}
		}

		if self.hovered != hovered {
			self.hovered = hovered;
		}

		EventResult::Pass
	}
}
