use glam::Vec2;

use crate::{
	layout::{WidgetID, WidgetMap},
	transform_stack::Transform,
	widget::WidgetObj,
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

pub struct MouseWheelEvent {
	pub pos: Vec2,
	pub shift: Vec2,
}

pub enum Event {
	MouseDown(MouseDownEvent),
	MouseMotion(MouseMotionEvent),
	MouseUp(MouseUpEvent),
	MouseWheel(MouseWheelEvent),
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
			Event::MouseWheel(evt) => self.test_transform_pos(transform, &evt.pos),
		}
	}
}

pub struct CallbackData<'a> {
	pub obj: &'a mut dyn WidgetObj,
	pub widgets: &'a WidgetMap,
	pub widget_id: WidgetID,
	pub node_id: taffy::NodeId,
}

impl CallbackData<'_> {
	pub fn call_on_widget<WIDGET, FUNC>(&self, widget_id: WidgetID, func: FUNC)
	where
		WIDGET: WidgetObj,
		FUNC: FnOnce(&mut WIDGET),
	{
		let Some(widget) = self.widgets.get(widget_id) else {
			debug_assert!(false);
			return;
		};

		let mut lock = widget.lock().unwrap();
		let m = lock.obj.get_as_mut::<WIDGET>();

		func(m);
	}
}

pub type MouseEnterCallback = Box<dyn Fn(&mut CallbackData)>;
pub type MouseLeaveCallback = Box<dyn Fn(&mut CallbackData)>;
pub type MouseClickCallback = Box<dyn Fn(&mut CallbackData)>;

pub enum EventListener {
	MouseEnter(MouseEnterCallback),
	MouseLeave(MouseLeaveCallback),
	MouseClick(MouseClickCallback),
}
