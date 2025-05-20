use std::collections::HashMap;

use taffy::{
	AlignContent, AlignItems, AlignSelf, BoxSizing, FlexDirection, FlexWrap, JustifyContent,
	JustifySelf, Overflow,
};

use crate::{
	drawing::{self, GradientMode},
	layout::{Layout, WidgetID},
	renderers::text::{FontWeight, HorizontalAlign},
	widget::{
		div::Div,
		rectangle::{Rectangle, RectangleParams},
		text::{TextLabel, TextParams},
	},
};

#[derive(Default)]
pub struct ParserResult {
	pub ids: HashMap<String, WidgetID>,
}

impl ParserResult {
	pub fn require_by_id(&self, id: &str) -> anyhow::Result<WidgetID> {
		match self.ids.get(id) {
			Some(id) => Ok(*id),
			None => anyhow::bail!("Widget by ID \"{}\" doesn't exist", id),
		}
	}
}

struct ParserContext<'a> {
	layout: &'a mut Layout,
	result: &'a mut ParserResult,
}

// Parses a color from a HTML hex string
pub fn parse_color(html_hex: &str) -> Option<drawing::Color> {
	if html_hex.len() == 7 {
		if let (Ok(r), Ok(g), Ok(b)) = (
			u8::from_str_radix(&html_hex[1..3], 16),
			u8::from_str_radix(&html_hex[3..5], 16),
			u8::from_str_radix(&html_hex[5..7], 16),
		) {
			return Some(drawing::Color::new(
				f32::from(r) / 255.,
				f32::from(g) / 255.,
				f32::from(b) / 255.,
				1.,
			));
		}
	} else if html_hex.len() == 9 {
		if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
			u8::from_str_radix(&html_hex[1..3], 16),
			u8::from_str_radix(&html_hex[3..5], 16),
			u8::from_str_radix(&html_hex[5..7], 16),
			u8::from_str_radix(&html_hex[7..9], 16),
		) {
			return Some(drawing::Color::new(
				f32::from(r) / 255.,
				f32::from(g) / 255.,
				f32::from(b) / 255.,
				f32::from(a) / 255.,
			));
		}
	}
	log::warn!("failed to parse color \"{}\"", html_hex);
	None
}

fn get_tag_by_name<'a>(
	node: roxmltree::Node<'a, 'a>,
	name: &str,
) -> Option<roxmltree::Node<'a, 'a>> {
	node
		.children()
		.find(|&child| child.tag_name().name() == name)
}

fn require_tag_by_name<'a>(
	node: roxmltree::Node<'a, 'a>,
	name: &str,
) -> anyhow::Result<roxmltree::Node<'a, 'a>> {
	get_tag_by_name(node, name).ok_or_else(|| anyhow::anyhow!("Tag \"{}\" not found", name))
}

fn print_invalid_attrib(key: &str, value: &str) {
	log::warn!("Invalid value {} in attribute {}", key, value);
}

fn print_invalid_value(value: &str) {
	log::warn!("Invalid value {}", value);
}

fn parse_val(value: &str) -> Option<f32> {
	let Ok(val) = value.parse::<f32>() else {
		print_invalid_value(value);
		return None;
	};
	Some(val)
}

fn parse_size_unit<T>(value: &str) -> Option<T>
where
	T: taffy::prelude::FromPercent + taffy::prelude::FromLength,
{
	if value.ends_with("%") {
		// percentage mode
		let Some(val_str) = value.split("%").next() else {
			print_invalid_value(value);
			return None;
		};

		let Ok(val) = val_str.parse::<f32>() else {
			print_invalid_value(value);
			return None;
		};

		Some(taffy::prelude::percent(val / 100.0))
	} else {
		// normal mode
		let Ok(val) = value.parse::<f32>() else {
			print_invalid_value(value);
			return None;
		};
		Some(taffy::prelude::length(val))
	}
}

fn style_from_node<'a>(node: roxmltree::Node<'a, 'a>) -> taffy::Style {
	let mut style = taffy::Style {
		..Default::default()
	};

	for attrib in node.attributes() {
		let (key, value) = (attrib.name(), attrib.value());

		match key {
			"margin_left" => {
				if let Some(dim) = parse_size_unit(value) {
					style.margin.left = dim;
				}
			}
			"margin_right" => {
				if let Some(dim) = parse_size_unit(value) {
					style.margin.right = dim;
				}
			}
			"margin_top" => {
				if let Some(dim) = parse_size_unit(value) {
					style.margin.top = dim;
				}
			}
			"margin_bottom" => {
				if let Some(dim) = parse_size_unit(value) {
					style.margin.bottom = dim;
				}
			}
			"padding_left" => {
				if let Some(dim) = parse_size_unit(value) {
					style.padding.left = dim;
				}
			}
			"padding_right" => {
				if let Some(dim) = parse_size_unit(value) {
					style.padding.right = dim;
				}
			}
			"padding_top" => {
				if let Some(dim) = parse_size_unit(value) {
					style.padding.top = dim;
				}
			}
			"padding_bottom" => {
				if let Some(dim) = parse_size_unit(value) {
					style.padding.bottom = dim;
				}
			}
			"margin" => {
				if let Some(dim) = parse_size_unit(value) {
					style.margin.left = dim;
					style.margin.right = dim;
					style.margin.top = dim;
					style.margin.bottom = dim;
				}
			}
			"padding" => {
				if let Some(dim) = parse_size_unit(value) {
					style.padding.left = dim;
					style.padding.right = dim;
					style.padding.top = dim;
					style.padding.bottom = dim;
				}
			}
			"overflow_x" => match value {
				"hidden" => style.overflow.x = Overflow::Hidden,
				"visible" => style.overflow.x = Overflow::Visible,
				"clip" => style.overflow.x = Overflow::Clip,
				"scroll" => style.overflow.x = Overflow::Scroll,
				_ => {
					print_invalid_attrib(key, value);
				}
			},
			"overflow_y" => match value {
				"hidden" => style.overflow.y = Overflow::Hidden,
				"visible" => style.overflow.y = Overflow::Visible,
				"clip" => style.overflow.y = Overflow::Clip,
				"scroll" => style.overflow.y = Overflow::Scroll,
				_ => {
					print_invalid_attrib(key, value);
				}
			},
			"min_width" => {
				if let Some(dim) = parse_size_unit(value) {
					style.min_size.width = dim;
				}
			}
			"min_height" => {
				if let Some(dim) = parse_size_unit(value) {
					style.min_size.height = dim;
				}
			}
			"max_width" => {
				if let Some(dim) = parse_size_unit(value) {
					style.max_size.width = dim;
				}
			}
			"max_height" => {
				if let Some(dim) = parse_size_unit(value) {
					style.max_size.height = dim;
				}
			}
			"width" => {
				if let Some(dim) = parse_size_unit(value) {
					style.size.width = dim;
				}
			}
			"height" => {
				if let Some(dim) = parse_size_unit(value) {
					style.size.height = dim;
				}
			}
			"gap" => {
				if let Some(val) = parse_size_unit(value) {
					style.gap = val;
				}
			}
			"flex_basis" => {
				if let Some(val) = parse_size_unit(value) {
					style.flex_basis = val;
				}
			}
			"flex_grow" => {
				if let Some(val) = parse_val(value) {
					style.flex_grow = val;
				}
			}
			"flex_shrink" => {
				if let Some(val) = parse_val(value) {
					style.flex_shrink = val;
				}
			}
			"position" => match value {
				"absolute" => style.position = taffy::Position::Absolute,
				"relative" => style.position = taffy::Position::Relative,
				_ => {
					print_invalid_attrib(key, value);
				}
			},
			"box_sizing" => match value {
				"border_box" => style.box_sizing = BoxSizing::BorderBox,
				"content_box" => style.box_sizing = BoxSizing::ContentBox,
				_ => {
					print_invalid_attrib(key, value);
				}
			},
			"align_self" => match value {
				"baseline" => style.align_self = Some(AlignSelf::Baseline),
				"center" => style.align_self = Some(AlignSelf::Center),
				"end" => style.align_self = Some(AlignSelf::End),
				"flex_end" => style.align_self = Some(AlignSelf::FlexEnd),
				"flex_start" => style.align_self = Some(AlignSelf::FlexStart),
				"start" => style.align_self = Some(AlignSelf::Start),
				"stretch" => style.align_self = Some(AlignSelf::Stretch),
				_ => {
					print_invalid_attrib(key, value);
				}
			},
			"justify_self" => match value {
				"center" => style.justify_self = Some(JustifySelf::Center),
				"end" => style.justify_self = Some(JustifySelf::End),
				"flex_end" => style.justify_self = Some(JustifySelf::FlexEnd),
				"flex_start" => style.justify_self = Some(JustifySelf::FlexStart),
				"start" => style.justify_self = Some(JustifySelf::Start),
				"stretch" => style.justify_self = Some(JustifySelf::Stretch),
				_ => {
					print_invalid_attrib(key, value);
				}
			},
			"align_items" => match value {
				"baseline" => style.align_items = Some(AlignItems::Baseline),
				"center" => style.align_items = Some(AlignItems::Center),
				"end" => style.align_items = Some(AlignItems::End),
				"flex_end" => style.align_items = Some(AlignItems::FlexEnd),
				"flex_start" => style.align_items = Some(AlignItems::FlexStart),
				"start" => style.align_items = Some(AlignItems::Start),
				"stretch" => style.align_items = Some(AlignItems::Stretch),
				_ => {
					print_invalid_attrib(key, value);
				}
			},
			"align_content" => match value {
				"center" => style.align_content = Some(AlignContent::Center),
				"end" => style.align_content = Some(AlignContent::End),
				"flex_end" => style.align_content = Some(AlignContent::FlexEnd),
				"flex_start" => style.align_content = Some(AlignContent::FlexStart),
				"space_around" => style.align_content = Some(AlignContent::SpaceAround),
				"space_between" => style.align_content = Some(AlignContent::SpaceBetween),
				"space_evenly" => style.align_content = Some(AlignContent::SpaceEvenly),
				"start" => style.align_content = Some(AlignContent::Start),
				"stretch" => style.align_content = Some(AlignContent::Stretch),
				_ => {
					print_invalid_attrib(key, value);
				}
			},
			"justify_content" => match value {
				"center" => style.justify_content = Some(JustifyContent::Center),
				"end" => style.justify_content = Some(JustifyContent::End),
				"flex_end" => style.justify_content = Some(JustifyContent::FlexEnd),
				"flex_start" => style.justify_content = Some(JustifyContent::FlexStart),
				"space_around" => style.justify_content = Some(JustifyContent::SpaceAround),
				"space_between" => style.justify_content = Some(JustifyContent::SpaceBetween),
				"space_evenly" => style.justify_content = Some(JustifyContent::SpaceEvenly),
				"start" => style.justify_content = Some(JustifyContent::Start),
				"stretch" => style.justify_content = Some(JustifyContent::Stretch),
				_ => {
					print_invalid_attrib(key, value);
				}
			},
			"flex_wrap" => match value {
				"wrap" => style.flex_wrap = FlexWrap::Wrap,
				"no_wrap" => style.flex_wrap = FlexWrap::NoWrap,
				"wrap_reverse" => style.flex_wrap = FlexWrap::WrapReverse,
				_ => {}
			},
			"flex_direction" => match value {
				"column_reverse" => style.flex_direction = FlexDirection::ColumnReverse,
				"column" => style.flex_direction = FlexDirection::Column,
				"row_reverse" => style.flex_direction = FlexDirection::RowReverse,
				"row" => style.flex_direction = FlexDirection::Row,
				_ => {
					print_invalid_attrib(key, value);
				}
			},
			_ => {}
		}
	}

	style
}

fn parse_widget_div<'a>(
	ctx: &mut ParserContext,
	node: roxmltree::Node<'a, 'a>,
	parent_id: WidgetID,
) -> anyhow::Result<()> {
	let (new_id, _) = ctx
		.layout
		.add_child(parent_id, Div::create()?, style_from_node(node))?;

	parse_universal(ctx, node, new_id)?;
	parse_children(ctx, node, new_id)?;

	Ok(())
}

fn parse_widget_rectangle<'a>(
	ctx: &mut ParserContext,
	node: roxmltree::Node<'a, 'a>,
	parent_id: WidgetID,
) -> anyhow::Result<()> {
	let mut params = RectangleParams::default();

	for attrib in node.attributes() {
		let (key, value) = (attrib.name(), attrib.value());

		#[allow(clippy::single_match)]
		match key {
			"color" => {
				if let Some(color) = parse_color(value) {
					params.color = color;
				} else {
					print_invalid_attrib(key, value);
				}
			}
			"color2" => {
				if let Some(color) = parse_color(value) {
					params.color2 = color;
				} else {
					print_invalid_attrib(key, value);
				}
			}
			"gradient" => {
				params.gradient = match value {
					"horizontal" => GradientMode::Horizontal,
					"vertical" => GradientMode::Vertical,
					"radial" => GradientMode::Radial,
					"none" => GradientMode::None,
					_ => {
						print_invalid_attrib(key, value);
						GradientMode::None
					}
				}
			}
			"round" => {
				params.round = value.parse().unwrap_or_else(|_| {
					print_invalid_attrib(key, value);
					0.0
				});
			}
			"border" => {
				params.border = value.parse().unwrap_or_else(|_| {
					print_invalid_attrib(key, value);
					0.0
				});
			}
			"border_color" => {
				if let Some(color) = parse_color(value) {
					params.border_color = color;
				} else {
					print_invalid_attrib(key, value);
				}
			}
			_ => {}
		}
	}

	let (new_id, _) =
		ctx
			.layout
			.add_child(parent_id, Rectangle::create(params)?, style_from_node(node))?;

	parse_universal(ctx, node, new_id)?;
	parse_children(ctx, node, new_id)?;

	Ok(())
}

fn parse_widget_label<'a>(
	ctx: &mut ParserContext,
	node: roxmltree::Node<'a, 'a>,
	parent_id: WidgetID,
) -> anyhow::Result<()> {
	let mut params = TextParams::default();

	for attrib in node.attributes() {
		let (key, value) = (attrib.name(), attrib.value());

		#[allow(clippy::single_match)]
		match key {
			"text" => {
				params.content = String::from(value);
			}
			"color" => {
				if let Some(color) = parse_color(value) {
					params.style.color = Some(color);
				}
			}
			"align" => match value {
				"left" => params.style.align = Some(HorizontalAlign::Left),
				"right" => params.style.align = Some(HorizontalAlign::Right),
				"center" => params.style.align = Some(HorizontalAlign::Center),
				"justified" => params.style.align = Some(HorizontalAlign::Justified),
				"end" => params.style.align = Some(HorizontalAlign::End),
				_ => {
					print_invalid_attrib(key, value);
				}
			},
			"weight" => match value {
				"normal" => params.style.weight = Some(FontWeight::Normal),
				"bold" => params.style.weight = Some(FontWeight::Bold),
				_ => {
					print_invalid_attrib(key, value);
				}
			},
			"size" => {
				if let Ok(size) = value.parse::<f32>() {
					params.style.size = Some(size);
				} else {
					print_invalid_attrib(key, value);
				}
			}
			_ => {}
		}
	}

	let (new_id, _) =
		ctx
			.layout
			.add_child(parent_id, TextLabel::create(params)?, style_from_node(node))?;

	parse_universal(ctx, node, new_id)?;
	parse_children(ctx, node, new_id)?;

	Ok(())
}

fn parse_universal<'a>(
	ctx: &mut ParserContext,
	node: roxmltree::Node<'a, 'a>,
	widget_id: WidgetID,
) -> anyhow::Result<()> {
	for attrib in node.attributes() {
		let (key, value) = (attrib.name(), attrib.value());

		#[allow(clippy::single_match)]
		match key {
			"id" => {
				// Attach a specific widget to name-ID map (just like getElementById)
				if ctx
					.result
					.ids
					.insert(String::from(value), widget_id)
					.is_some()
				{
					log::warn!("duplicate ID \"{}\" in the same layout file!", value);
				}
			}
			_ => {}
		}
	}
	Ok(())
}

fn parse_children<'a>(
	ctx: &mut ParserContext,
	node: roxmltree::Node<'a, 'a>,
	parent_id: WidgetID,
) -> anyhow::Result<()> {
	for child_node in node.children() {
		match child_node.tag_name().name() {
			"div" => {
				parse_widget_div(ctx, child_node, parent_id)?;
			}
			"rectangle" => {
				parse_widget_rectangle(ctx, child_node, parent_id)?;
			}
			"label" => {
				parse_widget_label(ctx, child_node, parent_id)?;
			}
			_ => {}
		}
	}
	Ok(())
}

pub fn parse(layout: &mut Layout, parent_id: WidgetID, xml: &str) -> anyhow::Result<ParserResult> {
	let mut result = ParserResult::default();

	let mut ctx = ParserContext {
		layout,
		result: &mut result,
	};

	let opt = roxmltree::ParsingOptions {
		allow_dtd: true,
		..Default::default()
	};

	let doc = roxmltree::Document::parse_with_options(xml, opt)?;
	let root = doc.root();
	let tag_layout = require_tag_by_name(root, "layout")?;
	let tag_elements = require_tag_by_name(tag_layout, "elements")?;

	parse_children(&mut ctx, tag_elements, parent_id)?;

	Ok(result)
}
