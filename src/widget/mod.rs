use glam::Vec2;

use super::drawing::RenderPrimitive;
use crate::{
	any::AnyTrait,
	drawing,
	event::{CallbackData, Event, EventListener},
	layout::{Layout, WidgetID, WidgetMap},
	transform_stack::TransformStack,
};

pub mod div;
pub mod rectangle;
pub mod text;

pub struct WidgetState {
	pub hovered: bool,
	pub pressed: bool,
	pub event_listeners: Vec<EventListener>,
	pub obj: Box<dyn WidgetObj>,
}

impl WidgetState {
	fn new(obj: Box<dyn WidgetObj>) -> anyhow::Result<WidgetState> {
		Ok(Self {
			hovered: false,
			pressed: false,
			event_listeners: Vec::new(),
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
	pub widgets: &'a WidgetMap,
	pub tree: &'a taffy::TaffyTree<WidgetID>,
	pub transform_stack: &'a TransformStack,
}

pub enum EventResult {
	Pass,
	Consumed,
	Outside,
}

impl dyn WidgetObj {
	pub fn draw_all(&mut self, state: &mut DrawState, params: &DrawParams) {
		self.draw(state, params);

		self.draw_scrollbars(state, params);
	}

	fn draw_scrollbars(&mut self, state: &mut DrawState, params: &DrawParams) {
		let enabled_horiz = params.style.overflow.x == taffy::Overflow::Scroll;
		let enabled_vert = params.style.overflow.y == taffy::Overflow::Scroll;

		if !enabled_horiz && !enabled_vert {
			return;
		}

		let l = params.taffy_layout;
		let overflow = Vec2::new(l.scroll_width(), l.scroll_height());

		if overflow.x == 0.0 && overflow.y == 0.0 {
			return; // not overflowing
		}

		let l = params.taffy_layout;

		let size = Vec2::new(l.content_size.width, l.content_size.height);

		//println!("content {:?} overflow {:?}", size, overflow);

		let handle_size = 1.0 - (overflow / size);
		//println!("handle sizes {:?}", handle_size);

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
		if enabled_horiz && handle_size.x < 1.0 {
			state.primitives.push(drawing::RenderPrimitive::Rectangle(
				drawing::Boundary::from_pos_size(
					&Vec2::new(
						transform.pos.x,
						transform.pos.y + transform.dim.y - thickness - margin,
					),
					&Vec2::new(transform.dim.x * handle_size.x, thickness),
				),
				rect_params,
			));
		}

		// Vertical handle
		if enabled_vert && handle_size.y < 1.0 {
			state.primitives.push(drawing::RenderPrimitive::Rectangle(
				drawing::Boundary::from_pos_size(
					&Vec2::new(
						transform.pos.x + transform.dim.x - thickness - margin,
						transform.pos.y,
					),
					&Vec2::new(thickness, transform.dim.y * handle_size.y),
				),
				rect_params,
			));
		}
	}

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
