use taffy::Style;

use super::{
	drawing::{self, RenderPrimitive},
	layout::WidgetHandle,
};

pub mod div;
pub mod rectangle;

pub struct WidgetData {
	pub node: taffy::NodeId,
	pub children: Vec<WidgetHandle>,
	pub parent: WidgetHandle,
}

impl WidgetData {
	fn boundary(&self) -> drawing::Boundary {
		// Todo
		drawing::Boundary {
			x: 0.0,
			y: 0.0,
			w: 128.0,
			h: 32.0,
		}
	}

	fn from_params(params: &mut InitParams) -> anyhow::Result<WidgetData> {
		let node = params.tree.new_leaf(Style {
			..Default::default()
		})?;

		Ok(Self {
			children: Vec::new(),
			parent: WidgetHandle {
				idx: 0,
				generation: u64::MAX, // Unset by default
			},
			node,
		})
	}
}

pub struct InitParams<'a> {
	pub tree: &'a mut taffy::TaffyTree<()>,
}

pub struct DrawParams<'a> {
	pub primitives: &'a mut Vec<RenderPrimitive>,
}

pub trait Widget {
	fn data(&self) -> &WidgetData;
	fn data_mut(&mut self) -> &mut WidgetData;
	fn draw(&self, params: &mut DrawParams);
}
