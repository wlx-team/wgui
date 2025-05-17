use tiny_skia::{Paint, Pixmap};
use wgui::{
	drawing::{self, Color, RenderPrimitive},
	glam::Vec2,
	layout::Layout,
	taffy::{self, AlignContent, AlignItems, FlexDirection, prelude::length},
	text::{FontWeight, HorizontalAlign, TextStyle},
	widget::{
		rectangle::{Rectangle, RectangleParams},
		text::{TextLabel, TextParams},
	},
};

pub struct Testbed {
	layout: Layout,
}

impl Testbed {
	pub fn new() -> anyhow::Result<Self> {
		let mut layout = Layout::new()?;

		let container = layout.add_child(
			layout.root,
			Rectangle::new(RectangleParams {
				color: drawing::Color([0.8, 0.8, 0.8, 1.0]),
			})?,
			taffy::Style {
				flex_direction: FlexDirection::Row,
				size: taffy::Size::percent(1.0),
				..Default::default()
			},
		)?;

		let first_rect = layout.add_child(
			container,
			Rectangle::new(RectangleParams {
				color: drawing::Color([0.8, 0.5, 0.2, 1.0]),
			})?,
			taffy::Style {
				size: taffy::Size::percent(1.0),
				flex_wrap: taffy::FlexWrap::Wrap,
				align_items: Some(AlignItems::FlexStart),
				align_content: Some(AlignContent::FlexStart),
				..Default::default()
			},
		)?;

		let second_rect = layout.add_child(
			container,
			Rectangle::new(RectangleParams {
				color: drawing::Color([0.5, 0.8, 0.2, 1.0]),
			})?,
			taffy::Style {
				size: taffy::Size::percent(1.0),
				..Default::default()
			},
		)?;

		let third_rect = layout.add_child(
			container,
			Rectangle::new(RectangleParams {
				color: drawing::Color([0.2, 0.5, 0.8, 1.0]),
			})?,
			taffy::Style {
				size: taffy::Size::percent(1.0),
				..Default::default()
			},
		)?;

		// add some boxes to a first rect

		let align = [
			HorizontalAlign::Left,
			HorizontalAlign::Center,
			HorizontalAlign::Right,
		];

		for i in 0..40 {
			let rect = layout.add_child(
				first_rect,
				Rectangle::new(RectangleParams {
					color: drawing::Color([1.0 - (i as f32 / 40.0), 0.0, i as f32 / 40.0, 1.0]),
				})?,
				taffy::Style {
					size: taffy::Size {
						width: length(32.0 + i as f32 * 2.0),
						height: length(32.0),
					},
					margin: taffy::Rect::length(8.0),
					..Default::default()
				},
			)?;
			let _text = layout.add_child(
				rect,
				TextLabel::new(TextParams {
					content: format!("{i:#02x}"),
					style: TextStyle {
						align: Some(align[i % 3]),
						weight: Some(FontWeight::Bold),
						size: Some(12.),
						color: Some(drawing::Color([0.0, 1.0, 1.0 - (i as f32 / 40.0), 1.0])),
						..Default::default()
					},
				})?,
				taffy::Style {
					size: taffy::Size::percent(1.0),
					margin: taffy::Rect::length(4.0),
					..Default::default()
				},
			)?;
		}

		let _text = layout.add_child(
			second_rect,
			TextLabel::new(TextParams {
				content: "Multi line test #1\nThis is aligned to the left.".into(),
				style: TextStyle {
					align: Some(HorizontalAlign::Left),
					size: Some(20.),
					color: Some(Color([0.0, 0.0, 0.0, 0.5])),
					..Default::default()
				},
			})?,
			taffy::Style {
				size: taffy::Size::percent(1.0),
				margin: taffy::Rect::length(4.0),
				..Default::default()
			},
		)?;

		let _text = layout.add_child(
			second_rect,
			TextLabel::new(TextParams {
				content: "Multi line test #2\nThis is aligned to the left.".into(),
				style: TextStyle {
					align: Some(HorizontalAlign::Right),
					size: Some(20.),
					color: Some(Color([0.0, 0.0, 0.0, 0.5])),
					..Default::default()
				},
			})?,
			taffy::Style {
				size: taffy::Size::percent(1.0),
				margin: taffy::Rect::length(4.0),
				..Default::default()
			},
		)?;

		let _text = layout.add_child(
			second_rect,
			TextLabel::new(TextParams {
				content: "Multi line test #3\nThis is aligned to the center.\nThe longer lines are still alinged to the center.".into(),
				style: TextStyle {
                    align: Some(HorizontalAlign::Center),
					size: Some(20.),
					color: Some(Color([0.0, 0.0, 0.0, 0.7])),
					..Default::default()
				},
			})?,
			taffy::Style {
				size: taffy::Size::percent(1.0),
				margin: taffy::Rect::length(4.0),
				..Default::default()
			},
		)?;

		let _text = layout.add_child(
			second_rect,
			TextLabel::new(TextParams {
				content: "Multi line test #4\nThis is justified alignment.\nThe longer lines are the same length as the shorter lines.".into(),
				style: TextStyle {
                    align: Some(HorizontalAlign::Justified),
					size: Some(20.),
					color: Some(Color([0.0, 0.0, 0.0, 0.7])),
					..Default::default()
				},
			})?,
			taffy::Style {
				size: taffy::Size::percent(1.0),
				margin: taffy::Rect::length(4.0),
				..Default::default()
			},
		)?;

		let _text = layout.add_child(
			second_rect,
			TextLabel::new(TextParams {
				content: "Multi line test #3\nThis is aligned to the left.".into(),
				style: TextStyle {
					size: Some(20.),
					color: Some(Color([0.0, 0.0, 0.0, 0.5])),
					..Default::default()
				},
			})?,
			taffy::Style {
				size: taffy::Size::percent(1.0),
				margin: taffy::Rect::length(4.0),
				..Default::default()
			},
		)?;

		let _third_text = layout.add_child(
			third_rect,
			TextLabel::new(TextParams {
				content: "Word wrap test:\n\nLorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".into(),
				style: TextStyle {
          wrap: true,
					size: Some(20.),
					color: Some(Color([1.0, 1.0, 0.0, 1.0])),
					..Default::default()
				},
			})?,
			taffy::Style {
				size: taffy::Size::percent(1.0),
				margin: taffy::Rect::length(4.0),
				..Default::default()
			},
		)?;

		Ok(Self { layout })
	}

	pub fn update(&mut self, width: f32, height: f32) -> anyhow::Result<()> {
		self.layout.update(Vec2::new(width, height))?;
		Ok(())
	}

	pub fn draw(&self, pixmap: &mut Pixmap, transform: &tiny_skia::Transform) -> anyhow::Result<()> {
		let primitives = wgui::drawing::draw(&self.layout)?;
		for primitive in primitives {
			match primitive {
				RenderPrimitive::Rectangle(boundary, rectangle) => {
					let mut paint = Paint::default();
					let col = &rectangle.color.0;
					paint.set_color(tiny_skia::Color::from_rgba(col[0], col[1], col[2], col[3]).unwrap());

					pixmap.fill_rect(
						tiny_skia::Rect::from_xywh(boundary.x, boundary.y, boundary.w, boundary.h).unwrap(),
						&paint,
						*transform,
						None,
					);
				}
				RenderPrimitive::Text(boundary, mut text) => {
					text.draw(boundary, |x, y, w, h, color| {
						let mut paint = Paint::default();
						let col = color.0;
						paint.set_color(tiny_skia::Color::from_rgba(col[0], col[1], col[2], col[3]).unwrap());

						pixmap.fill_rect(
							tiny_skia::Rect::from_xywh(x as _, y as _, w as _, h as _).unwrap(),
							&paint,
							*transform,
							None,
						);
					});
				}
				RenderPrimitive::Image(_boundary, _image) => todo!(),
			}
		}

		Ok(())
	}
}
