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

pub struct DrawParams<'a> {
	pub layout: &'a Layout,
	pub primitives: &'a mut Vec<RenderPrimitive>,
	pub transform_stack: &'a mut TransformStack,
}

pub trait WidgetObj: AnyTrait {
	fn draw(&mut self, params: &mut DrawParams);
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
