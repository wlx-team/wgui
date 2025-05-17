use slotmap::Key;

use super::drawing::RenderPrimitive;
use crate::{
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
}

impl WidgetData {
	fn new() -> anyhow::Result<WidgetData> {
		Ok(Self {
			children: Vec::new(),
			parent: WidgetID::null(),    // Unset by default
			node: taffy::NodeId::new(0), // Unset by default
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
