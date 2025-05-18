use std::{cell::RefCell, rc::Rc};

use slotmap::Key;

use super::drawing::RenderPrimitive;
use crate::{
	any::AnyTrait,
	event::{CallbackData, Event, EventListener},
	layout::{Layout, WidgetID, WidgetMap},
	transform_stack::TransformStack,
};

pub mod div;
pub mod rectangle;
pub mod text;

#[derive(Default)]
pub struct WidgetState {
	pub hovered: bool,
	pub pressed: bool,
	pub event_listeners: Vec<EventListener>,
}

pub struct WidgetData {
	pub node: taffy::NodeId,
	pub children: Vec<WidgetID>,
	pub parent: WidgetID,

	pub state: Rc<RefCell<WidgetState>>,
}

impl WidgetData {
	fn new() -> anyhow::Result<WidgetData> {
		Ok(Self {
			children: Vec::new(),
			parent: WidgetID::null(),    // Unset by default
			node: taffy::NodeId::new(0), // Unset by default
			state: Rc::new(RefCell::new(WidgetState::default())),
		})
	}
}

pub struct DrawParams<'a> {
	pub current_widget: WidgetID,
	pub layout: &'a Layout,
	pub primitives: &'a mut Vec<RenderPrimitive>,
	pub transform_stack: &'a mut TransformStack,
}

pub trait Widget: AnyTrait {
	fn data(&self) -> &WidgetData;
	fn data_mut(&mut self) -> &mut WidgetData;
	fn draw(&self, params: &mut DrawParams);
	fn measure(
		&mut self,
		known_dimensions: taffy::Size<Option<f32>>,
		available_space: taffy::Size<taffy::AvailableSpace>,
	) -> taffy::Size<f32>;
}

pub struct EventParams<'a> {
	pub widgets: &'a WidgetMap,
	pub tree: &'a mut taffy::TaffyTree<WidgetID>,
	pub transform_stack: &'a TransformStack,
}

pub enum EventResult {
	Pass,
	Consumed,
	Outside,
}

impl dyn Widget {
	pub fn add_event_listener(&mut self, listener: EventListener) {
		self
			.data_mut()
			.state
			.borrow_mut()
			.event_listeners
			.push(listener);
	}

	pub fn process_event(
		&self,
		widget_id: WidgetID,
		event: &Event,
		params: &mut EventParams,
	) -> EventResult {
		let hovered = event.test_mouse_within_transform(params.transform_stack.get());
		let mut state = self.data().state.borrow_mut();

		match &event {
			Event::MouseDown(_) => {
				if hovered {
					state.pressed = true;
				}
			}
			Event::MouseUp(_) => {
				if state.pressed {
					state.pressed = false;
				}
			}
			_ => {}
		}

		let mut just_clicked = false;
		match &event {
			Event::MouseDown(_) => {
				if state.hovered {
					state.pressed = true;
				}
			}
			Event::MouseUp(_) => {
				if state.pressed {
					state.pressed = false;
					just_clicked = state.hovered;
				}
			}
			_ => {}
		}

		let callback_data = CallbackData {
			widgets: &params.widgets,
			widget_id,
		};

		for listener in &state.event_listeners {
			match listener {
				EventListener::MouseEnter(callback) => {
					if hovered && !state.hovered {
						callback(&callback_data);
					}
				}
				EventListener::MouseLeave(callback) => {
					if !hovered && !state.hovered {
						callback(&callback_data);
					}
				}
				EventListener::MouseClick(callback) => {
					if just_clicked {
						callback(&callback_data);
					}
				}
			}
		}

		if state.hovered != hovered {
			state.hovered = hovered;
			EventResult::Pass
		} else if hovered {
			EventResult::Pass
		} else {
			EventResult::Outside
		}
	}
}
