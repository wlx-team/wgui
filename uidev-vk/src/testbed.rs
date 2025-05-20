use wgui::{
	cosmic_text::Color,
	drawing::{self, RenderPrimitive},
	event::EventListener,
	gfx::cmd::GfxCommandBuffer,
	glam::Vec2,
	layout::Layout,
	renderers::text::{FONT_SYSTEM, SWASH_CACHE, TextArea, TextBounds, TextStyle},
	widget::text::TextLabel,
};

use crate::Goodies;

pub struct Testbed {
	pub layout: Layout,
	pub scale: f32,
}

const XML_PATH: &str = "res/testbed.xml";

impl Testbed {
	pub fn new() -> anyhow::Result<Self> {
		let mut layout = Layout::new()?;

		let parent = layout.root_widget;

		let res = wgui::parser::parse(
			&mut layout,
			parent,
			std::fs::read_to_string(XML_PATH).unwrap().as_str(),
		)?;

		use wgui::components::button;
		let my_div_parent = res.require_by_id("my_div_parent")?;

		// create some buttons for testing
		for i in 0..10 {
			let n = i as f32 / 10.0;
			button::construct(
				&mut layout,
				my_div_parent,
				button::Params {
					text: "I'm a button!",
					color: drawing::Color::new(1.0 - n, n * n, n, 1.0),
					..Default::default()
				},
			)?;
		}

		let button = button::construct(
			&mut layout,
			my_div_parent,
			button::Params {
				text: "Click me!!",
				color: drawing::Color::new(0.2, 0.2, 0.2, 1.0),
				size: Vec2::new(256.0, 64.0),
				text_style: TextStyle {
					size: Some(30.0),
					..Default::default()
				},
			},
		)?;

		layout.add_event_listener(
			button.body,
			EventListener::MouseClick(Box::new(move |data| {
				data.call_on_widget(button.text_id, |label: &mut TextLabel| {
					label.set_text("Congratulations!");
				});
			})),
		);

		Ok(Self { layout, scale: 1.5 })
	}

	pub fn update(&mut self, width: f32, height: f32) -> anyhow::Result<()> {
		self.layout.update(Vec2::new(width, height))?;
		Ok(())
	}

	pub fn draw(&self, cmd_buf: &mut GfxCommandBuffer, goodies: &mut Goodies) -> anyhow::Result<()> {
		let mut text_areas = vec![];

		let primitives = wgui::drawing::draw(&self.layout)?;
		for primitive in primitives.iter() {
			match primitive {
				RenderPrimitive::Rectangle(boundary, rectangle) => {
					goodies
						.rect_renderer
						.add_rect(*boundary, *rectangle, self.scale, 0.0);
				}
				RenderPrimitive::Text(boundary, text) => {
					text_areas.push(TextArea {
						buffer: text.get_buffer(),
						left: boundary.x * self.scale,
						top: boundary.y * self.scale,
						bounds: TextBounds::default(), //FIXME: just using boundary coords here doesn't work
						scale: self.scale,
						default_color: Color::rgb(255, 0, 0),
						custom_glyphs: &[],
						depth: 0.0, //FIXME: add depth info
					});
				}
				RenderPrimitive::Image(_boundary, _image) => todo!(),
			}
		}

		goodies
			.rect_renderer
			.render(&mut goodies.viewport, cmd_buf)?;

		{
			let mut font_system = FONT_SYSTEM.lock().unwrap();
			let mut swash_cache = SWASH_CACHE.lock().unwrap();

			goodies.text_renderer.prepare(
				&mut font_system,
				&mut goodies.text_atlas,
				&goodies.viewport,
				text_areas,
				&mut swash_cache,
			)?;
		}

		goodies
			.text_renderer
			.render(&goodies.text_atlas, &mut goodies.viewport, cmd_buf)?;

		Ok(())
	}
}
