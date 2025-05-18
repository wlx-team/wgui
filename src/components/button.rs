use std::sync::Arc;

use taffy::{
	AlignItems, JustifyContent,
	prelude::{length, percent},
};

use crate::{
	drawing::{self, Color},
	event::EventListener,
	layout::{Layout, WidgetID},
	renderers::text::{FontWeight, TextStyle},
	widget::{
		rectangle::{Rectangle, RectangleParams},
		text::{TextLabel, TextParams},
	},
};

pub struct Params<'a> {
	pub text: &'a str,
	pub color: drawing::Color,
}

impl Default for Params<'_> {
	fn default() -> Self {
		Self {
			text: "Text",
			color: drawing::Color::new(1.0, 1.0, 1.0, 1.0),
		}
	}
}

pub struct Button {
	color: drawing::Color,
}

pub fn construct(
	layout: &mut Layout,
	parent: WidgetID,
	params: Params,
) -> anyhow::Result<Arc<Button>> {
	// simulate a border because we don't have it yet
	let (outer_border, _) = layout.add_child(
		parent,
		Rectangle::create(RectangleParams::default())?,
		taffy::Style {
			size: taffy::Size {
				width: length(128.0),
				height: length(32.0),
			},
			padding: length(1.0),
			..Default::default()
		},
	)?;

	let (inner_bg, _) = layout.add_child(
		outer_border,
		Rectangle::create(RectangleParams {
			color: params.color,
			..Default::default()
		})?,
		taffy::Style {
			size: taffy::Size {
				width: percent(1.0),
				height: percent(1.0),
			},
			align_items: Some(AlignItems::Center),
			justify_content: Some(JustifyContent::Center),
			..Default::default()
		},
	)?;

	let light_text = (params.color.r + params.color.g + params.color.b) < 1.5;

	layout.add_child(
		inner_bg,
		TextLabel::create(TextParams {
			content: String::from(params.text),
			style: TextStyle {
				weight: Some(FontWeight::Bold),
				color: Some(if light_text {
					Color::new(1.0, 1.0, 1.0, 1.0)
				} else {
					Color::new(0.0, 0.0, 0.0, 1.0)
				}),
				..Default::default()
			},
		})?,
		taffy::Style {
			..Default::default()
		},
	)?;

	let mut widget = layout.widget_states.get(inner_bg).unwrap().lock().unwrap();

	let button = Arc::new(Button {
		color: params.color,
	});

	// Highlight background on mouse enter
	{
		let button = button.clone();
		widget.add_event_listener(EventListener::MouseEnter(Box::new(move |data| {
			let rect = data.obj.get_as_mut::<Rectangle>();
			rect.params.color.r = button.color.r + 0.2;
			rect.params.color.g = button.color.g + 0.2;
			rect.params.color.b = button.color.b + 0.2;

			// set border color to white
			let mut outer = data.widgets.get(outer_border).unwrap().lock().unwrap();
			let outer_rect = outer.obj.get_as_mut::<Rectangle>();
			outer_rect.params.color = Color::new(1.0, 1.0, 1.0, 1.0);
		})));
	}

	// Bring back old color on mouse leave
	{
		let button = button.clone();
		widget.add_event_listener(EventListener::MouseLeave(Box::new(move |data| {
			let rect = data.obj.get_as_mut::<Rectangle>();
			rect.params.color = button.color;

			// restore border color
			let mut outer = data.widgets.get(outer_border).unwrap().lock().unwrap();
			let outer_rect = outer.obj.get_as_mut::<Rectangle>();
			outer_rect.params.color = Color::new(0.0, 0.0, 0.0, 1.0);
		})));
	}

	Ok(button)
}
