use taffy::{
	AlignItems, JustifyContent,
	prelude::{length, percent},
};

use crate::{
	drawing::{self, Color},
	layout::{Layout, WidgetID},
	text::{FontWeight, TextStyle},
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
			color: drawing::Color([1.0, 1.0, 1.0, 1.0]),
		}
	}
}

pub struct Button {
	// to be filled later
}

pub fn construct(layout: &mut Layout, parent: WidgetID, params: Params) -> anyhow::Result<Button> {
	// simulate a border because we don't have it yet
	let outer_border = layout.add_child(
		parent,
		Rectangle::new(RectangleParams {
			color: Color([0.0, 0.0, 0.0, 1.0]),
		})?,
		taffy::Style {
			size: taffy::Size {
				width: length(128.0),
				height: length(32.0),
			},
			padding: length(1.0),
			..Default::default()
		},
	)?;

	let inner_bg = layout.add_child(
		outer_border,
		Rectangle::new(RectangleParams {
			color: params.color,
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

	let color = &params.color.0;

	let light_text = (color[0] + color[1] + color[2]) < 1.5;

	layout.add_child(
		inner_bg,
		TextLabel::new(TextParams {
			content: String::from(params.text),
			style: TextStyle {
				weight: Some(FontWeight::Bold),
				color: Some(if light_text {
					Color([1.0, 1.0, 1.0, 1.0])
				} else {
					Color([0.0, 0.0, 0.0, 1.0])
				}),
				..Default::default()
			},
		})?,
		taffy::Style {
			..Default::default()
		},
	)?;

	Ok(Button {})
}
