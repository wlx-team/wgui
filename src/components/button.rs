use std::sync::Arc;

use glam::Vec2;
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
	pub size: Vec2,
	pub text_style: TextStyle,
}

impl Default for Params<'_> {
	fn default() -> Self {
		Self {
			text: "Text",
			color: drawing::Color::new(1.0, 1.0, 1.0, 1.0),
			size: Vec2::new(128.0, 32.0),
			text_style: TextStyle::default(),
		}
	}
}

pub struct Button {
	color: drawing::Color,
	pub body: WidgetID,    // Rectangle
	pub text_id: WidgetID, // Text
}

pub fn construct(
	layout: &mut Layout,
	parent: WidgetID,
	params: Params,
) -> anyhow::Result<Arc<Button>> {
	let (rect_id, _) = layout.add_child(
		parent,
		Rectangle::create(RectangleParams {
			color: params.color,
			..Default::default()
		})?,
		taffy::Style {
			size: taffy::Size {
				width: length(params.size.x),
				height: length(params.size.y),
			},
			align_items: Some(AlignItems::Center),
			justify_content: Some(JustifyContent::Center),
			padding: length(1.0),
			..Default::default()
		},
	)?;

	let light_text = (params.color.r + params.color.g + params.color.b) < 1.5;

	let (text_id, _) = layout.add_child(
		rect_id,
		TextLabel::create(TextParams {
			content: String::from(params.text),
			style: TextStyle {
				weight: Some(FontWeight::Bold),
				color: Some(if light_text {
					Color::new(1.0, 1.0, 1.0, 1.0)
				} else {
					Color::new(0.0, 0.0, 0.0, 1.0)
				}),
				..params.text_style
			},
		})?,
		taffy::Style {
			..Default::default()
		},
	)?;

	let mut widget = layout.widget_states.get(rect_id).unwrap().lock().unwrap();

	let button = Arc::new(Button {
		body: rect_id,
		color: params.color,
		text_id,
	});

	// Highlight background on mouse enter
	{
		let button = button.clone();
		widget.add_event_listener(EventListener::MouseEnter(Box::new(move |data| {
			let rect = data.obj.get_as_mut::<Rectangle>();
			rect.params.color.r = button.color.r + 0.2;
			rect.params.color.g = button.color.g + 0.2;
			rect.params.color.b = button.color.b + 0.2;
			rect.params.border_color = Color::new(1.0, 1.0, 1.0, 1.0);
			rect.params.border = 1.0;
		})));
	}

	// Bring back old color on mouse leave
	{
		let button = button.clone();
		widget.add_event_listener(EventListener::MouseLeave(Box::new(move |data| {
			let rect = data.obj.get_as_mut::<Rectangle>();
			rect.params.color = button.color;
			rect.params.border = 0.0;
		})));
	}

	Ok(button)
}
