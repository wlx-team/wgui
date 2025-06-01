use std::{cell::RefCell, rc::Rc};

use glam::{Mat4, Vec3};
use wgui::{
	drawing::{self},
	event::EventListener,
	glam::Vec2,
	layout::Layout,
	renderer_vk::text::TextStyle,
};

pub struct Testbed {
	pub layout: Layout,
	pub scale: f32,
}

impl Testbed {
	pub fn new() -> anyhow::Result<Self> {
		const XML_PATH: &str = "res/testbed.xml";

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

		let rotation = Rc::new(RefCell::new(0.0));

		layout.add_event_listener(
			button.body,
			EventListener::MouseClick(Box::new(move |data| {
				button.set_text(data, "Congratulations!");
				*rotation.borrow_mut() += 0.15;
				data.widget_data.transform = Mat4::from_rotation_z(-*rotation.borrow())
					* Mat4::from_scale(Vec3::splat(1.0 + *rotation.borrow() + 0.1));
			})),
		);

		Ok(Self { layout, scale: 1.5 })
	}

	pub fn update(&mut self, width: f32, height: f32, timestep_alpha: f32) -> anyhow::Result<()> {
		self
			.layout
			.update(Vec2::new(width, height), timestep_alpha)?;
		Ok(())
	}
}
