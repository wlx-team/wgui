use crate::{
	event::{self, CallbackData, Event, EventListener},
	transform_stack::{Transform, TransformStack},
	widget::{self, EventParams},
};

use super::widget::{Widget, div::Div};
use glam::Vec2;
use slotmap::HopSlotMap;
use taffy::TaffyTree;

pub type WidgetID = slotmap::DefaultKey;
pub type BoxWidget = Box<dyn Widget>;
pub type WidgetMap = HopSlotMap<slotmap::DefaultKey, BoxWidget>;

pub struct Layout {
	pub tree: TaffyTree<WidgetID>,
	pub widgets: WidgetMap,
	pub root: WidgetID,

	pub prev_size: Vec2,
}

fn add_child_internal(
	tree: &mut taffy::TaffyTree<WidgetID>,
	vec: &mut WidgetMap,
	parent_widget: Option<WidgetID>,
	parent_node: Option<taffy::NodeId>,
	mut child: BoxWidget,
	style: taffy::Style,
) -> anyhow::Result<WidgetID> {
	let child_node = tree.new_leaf(style)?;
	if let Some(parent_node) = parent_node {
		tree.add_child(parent_node, child_node)?;
	}

	let child_id = vec.insert_with_key(|child_id| {
		let child_data = child.data_mut();
		child_data.node = child_node;

		if let Some(parent_widget) = parent_widget {
			child_data.parent = parent_widget;
		}

		tree.set_node_context(child_node, Some(child_id)).unwrap();

		child
	});

	if let Some(parent_widget) = parent_widget {
		let Some(parent_widget) = vec.get_mut(parent_widget) else {
			panic!("parent widget is invalid");
		};

		parent_widget.data_mut().children.push(child_id);
	}

	Ok(child_id)
}

impl Layout {
	pub fn add_child(
		&mut self,
		parent_widget_id: WidgetID,
		widget: BoxWidget,
		style: taffy::Style,
	) -> anyhow::Result<WidgetID> {
		let Some(parent_widget) = self.widgets.get(parent_widget_id) else {
			anyhow::bail!("invalid parent widget");
		};

		let parent_node = parent_widget.data().node;

		let child_id = add_child_internal(
			&mut self.tree,
			&mut self.widgets,
			Some(parent_widget_id),
			Some(parent_node),
			widget,
			style,
		)?;

		Ok(child_id)
	}

	fn push_event_widget(
		widgets: &WidgetMap,
		tree: &mut taffy::TaffyTree<WidgetID>,
		transform_stack: &mut TransformStack,
		widget_id: WidgetID,
		event: &event::Event,
	) -> anyhow::Result<()> {
		let Some(widget) = widgets.get(widget_id) else {
			debug_assert!(false);
			anyhow::bail!("invalid widget");
		};

		let data = widget.data();
		let l = tree.layout(data.node)?;

		let transform = Transform {
			pos: Vec2::new(l.location.x, l.location.y),
			dim: Vec2::new(l.size.width, l.size.height),
		};

		transform_stack.push(transform);

		let mut iter_children = true;

		match widget.process_event(
			widget_id,
			event,
			&mut EventParams {
				transform_stack,
				widgets,
				tree,
			},
		) {
			widget::EventResult::Pass => {
				// go on
			}
			widget::EventResult::Consumed => {
				todo!();
			}
			widget::EventResult::Outside => {
				iter_children = false;
			}
		}

		if iter_children {
			for child in &data.children {
				Layout::push_event_widget(widgets, tree, transform_stack, *child, event)?;
			}
		}

		transform_stack.pop();

		Ok(())
	}

	pub fn get_widget_as<T: 'static>(widgets: &WidgetMap, id: WidgetID) -> &T {
		let widget = widgets.get(id).unwrap();
		let any = (**widget).as_any();
		any.downcast_ref::<T>().unwrap()
	}

	pub fn get_widget_as_mut<T: 'static>(widgets: &mut WidgetMap, id: WidgetID) -> &T {
		let widget = widgets.get_mut(id).unwrap();
		let any = (**widget).as_any_mut();
		any.downcast_mut::<T>().unwrap()
	}

	pub fn push_event(&mut self, event: &event::Event) -> anyhow::Result<()> {
		let mut transform_stack = TransformStack::new();
		Layout::push_event_widget(
			&self.widgets,
			&mut self.tree,
			&mut transform_stack,
			self.root,
			event,
		)?;
		Ok(())
	}

	pub fn new() -> anyhow::Result<Self> {
		let mut tree = TaffyTree::new();

		let mut widgets = HopSlotMap::new();

		let root = add_child_internal(
			&mut tree,
			&mut widgets,
			None, /* no parent widget */
			None, /* no parent node */
			Div::new()?,
			taffy::Style {
				size: taffy::Size::percent(1.0),
				..Default::default()
			},
		)?;

		Ok(Self {
			tree,
			widgets,
			root,
			prev_size: Vec2::default(),
		})
	}

	pub fn update(&mut self, size: Vec2) -> anyhow::Result<()> {
		let root_node = self.widgets.get(self.root).unwrap().data().node;

		if self.tree.dirty(root_node)? || self.prev_size != size {
			println!("re-computing layout, size {}x{}", size.x, size.y);
			self.prev_size = size;
			self.tree.compute_layout(
				root_node,
				taffy::Size {
					width: taffy::AvailableSpace::Definite(size.x),
					height: taffy::AvailableSpace::Definite(size.y),
				},
				/*
								|known_dimensions, available_space, _node_id, node_context, _style| {
									if let Size {
										width: Some(width),
										height: Some(height),
									} = known_dimensions
									{
										return Size { width, height };
									}

									match node_context {
										None => Size::ZERO,
										Some(h) => {
											if let Some(w) = self.widgets.get_mut(h) {
												w.measure(known_dimensions, available_space)
											} else {
												Size::ZERO
											}
										}
									}
								},
				*/
			)?;
		}
		Ok(())
	}
}
