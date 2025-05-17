use slotmap::Key;

use super::drawing::RenderPrimitive;
use crate::{
	event::Event,
	layout::{Layout, WidgetID},
	transform_stack::TransformStack,
};

pub mod div;
pub mod rectangle;
pub mod text;

pub struct WidgetData {
	pub node: taffy::NodeId,
	pub children: Vec<WidgetID>,
	pub parent: WidgetID,

	// runtime variables
	pub hovered: bool,
}

impl WidgetData {
	fn new() -> anyhow::Result<WidgetData> {
		Ok(Self {
			children: Vec::new(),
			parent: WidgetID::null(),    // Unset by default
			node: taffy::NodeId::new(0), // Unset by default
			hovered: false,
		})
	}
}

pub struct DrawParams<'a> {
	pub current_widget: WidgetID,
	pub layout: &'a Layout,
	pub primitives: &'a mut Vec<RenderPrimitive>,
	pub transform_stack: &'a mut TransformStack,
}

pub trait Widget {
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
	pub transform_stack: &'a TransformStack,
}

pub enum EventResult {
	Pass,
	Consumed,
	Outside,
}

impl dyn Widget {
	pub fn process_event(&mut self, event: &Event, params: &EventParams) -> EventResult {
		let hovered = event.test_mouse_within_transform(params.transform_stack.get());

		let data = self.data_mut();
		if data.hovered != hovered {
			data.hovered = hovered;
			EventResult::Pass
		} else if hovered {
			EventResult::Pass
		} else {
			EventResult::Outside
		}
	}
}
