use std::rc::Rc;

use glam::Vec2;

use crate::{
	layout::{Layout, WidgetID, WidgetMap},
	transform_stack::Transform,
	widget::Widget,
};

// TODO: mouse index
pub struct MouseDownEvent {
	pub pos: Vec2,
}

pub struct MouseMotionEvent {
	pub pos: Vec2,
}

pub struct MouseUpEvent {
	pub pos: Vec2,
}

pub enum Event {
	MouseDown(MouseDownEvent),
	MouseMotion(MouseMotionEvent),
	MouseUp(MouseUpEvent),
}

impl Event {
	fn test_transform_pos(&self, transform: &Transform, pos: &Vec2) -> bool {
		pos.x >= transform.pos.x
			&& pos.x < transform.pos.x + transform.dim.x
			&& pos.y >= transform.pos.y
			&& pos.y < transform.pos.y + transform.dim.y
	}

	pub fn test_mouse_within_transform(&self, transform: &Transform) -> bool {
		match self {
			Event::MouseDown(evt) => self.test_transform_pos(transform, &evt.pos),
			Event::MouseMotion(evt) => self.test_transform_pos(transform, &evt.pos),
			Event::MouseUp(evt) => self.test_transform_pos(transform, &evt.pos),
		}
	}
}

pub struct CallbackData<'a> {
	pub widgets: &'a WidgetMap,
	pub widget_id: WidgetID,
}

pub type MouseEnterCallback = Box<dyn Fn(&CallbackData)>;
pub type MouseLeaveCallback = Box<dyn Fn(&CallbackData)>;
pub type MouseClickCallback = Box<dyn Fn(&CallbackData)>;

pub enum EventListener {
	MouseEnter(MouseEnterCallback),
	MouseLeave(MouseLeaveCallback),
	MouseClick(MouseClickCallback),
}
