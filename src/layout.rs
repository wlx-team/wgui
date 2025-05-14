use super::widget::{InitParams, Widget, div::Div};
use crate::gen_id;
use taffy::TaffyTree;

pub type BoxedWidget = Box<dyn Widget>;
gen_id!(WidgetVec, BoxedWidget, WidgetCell, WidgetHandle);

pub struct Layout {
	pub tree: TaffyTree<()>,
	pub widgets: WidgetVec,
	pub root: WidgetHandle,
}

fn add_child(
	vec: &mut WidgetVec,
	parent_widget: Option<WidgetHandle>,
	child: BoxedWidget,
) -> WidgetHandle {
	let child_handle = vec.add_with_post(child, |_, child| {
		let child_data = child.data_mut();

		if let Some(parent_widget) = parent_widget {
			child_data.parent = parent_widget;
		}
	});

	if let Some(parent_widget) = parent_widget {
		let Some(parent_widget) = vec.get_mut(&parent_widget) else {
			panic!("parent widget is invalid");
		};

		parent_widget.data_mut().children.push(child_handle);
	}

	child_handle
}

impl Layout {
	pub fn init_params(&mut self) -> InitParams {
		InitParams {
			tree: &mut self.tree,
		}
	}

	pub fn add_child(
		&mut self,
		parent_widget: Option<WidgetHandle>,
		widget: BoxedWidget,
	) -> WidgetHandle {
		add_child(&mut self.widgets, parent_widget, widget)
	}
}

impl Layout {
	pub fn new() -> anyhow::Result<Self> {
		let mut tree = TaffyTree::new();
		let mut widgets = WidgetVec::new();

		let root = add_child(
			&mut widgets,
			None, /* no parent because it's root */
			Box::new(Div::new(&mut InitParams { tree: &mut tree })?),
		);

		Ok(Self {
			tree,
			widgets,
			root,
		})
	}
}
