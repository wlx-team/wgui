use super::drawing::RenderPrimitive;
use crate::{
	any::AnyTrait,
	event::{CallbackData, Event, EventListener},
	layout::{Layout, WidgetID, WidgetStateMap},
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

impl WidgetState {
	fn new() -> anyhow::Result<WidgetState> {
		Ok(Self {
			hovered: false,
			pressed: false,
			event_listeners: Vec::new(),
		})
	}
}

pub struct DrawParams<'a> {
	pub layout: &'a Layout,
	pub primitives: &'a mut Vec<RenderPrimitive>,
	pub transform_stack: &'a mut TransformStack,
}

pub trait Widget: AnyTrait {
	fn state(&self) -> &WidgetState;
	fn state_mut(&mut self) -> &mut WidgetState;
	fn draw(&self, params: &mut DrawParams);
	fn measure(
		&mut self,
		known_dimensions: taffy::Size<Option<f32>>,
		available_space: taffy::Size<taffy::AvailableSpace>,
	) -> taffy::Size<f32>;
}

pub struct EventParams<'a> {
	pub widgets: &'a WidgetStateMap,
	pub tree: &'a taffy::TaffyTree<WidgetID>,
	pub transform_stack: &'a TransformStack,
}

pub enum EventResult {
	Pass,
	Consumed,
	Outside,
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

		match &event {
			Event::MouseDown(_) => {
				if hovered {
					self.pressed = true;
				}
			}
			Event::MouseUp(_) => {
				if self.pressed {
					self.pressed = false;
				}
			}
			_ => {}
		}

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

		let callback_data = CallbackData {
			widgets: params.widgets,
			widget_id,
			node_id,
		};

		for listener in &self.event_listeners {
			match listener {
				EventListener::MouseEnter(callback) => {
					if hovered && !self.hovered {
						callback(&callback_data);
					}
				}
				EventListener::MouseLeave(callback) => {
					if !hovered && !self.hovered {
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

		if self.hovered != hovered {
			self.hovered = hovered;
			EventResult::Pass
		} else if hovered {
			EventResult::Pass
		} else {
			EventResult::Outside
		}
	}
}
