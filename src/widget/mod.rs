use super::{drawing::RenderPrimitive, layout::WidgetHandle};
use crate::{layout::Layout, transform_stack::TransformStack};

pub mod div;
pub mod rectangle;

pub struct WidgetData {
	pub node: taffy::NodeId,
	pub children: Vec<WidgetHandle>,
	pub parent: WidgetHandle,
}

impl WidgetData {
	fn new() -> anyhow::Result<WidgetData> {
		Ok(Self {
			children: Vec::new(),
			parent: WidgetHandle {
				idx: 0,
				generation: u64::MAX, // Unset by default
			},
			node: taffy::NodeId::new(0), // Unset by default
		})
	}
}

pub struct DrawParams<'a> {
	pub layout: &'a Layout,
	pub primitives: &'a mut Vec<RenderPrimitive>,
	pub transform_stack: &'a mut TransformStack,
}

pub trait Widget {
	fn data(&self) -> &WidgetData;
	fn data_mut(&mut self) -> &mut WidgetData;
	fn draw(&self, params: &mut DrawParams);
}
