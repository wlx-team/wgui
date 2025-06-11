use wgui::{glam::Vec2, layout::Layout};

use crate::{assets, testbed::Testbed};

pub struct TestbedDashboard {
	pub layout: Layout,
}

impl TestbedDashboard {
	pub fn new() -> anyhow::Result<Self> {
		const XML_PATH: &str = "res/dashboard.xml";

		let mut layout = Layout::new(Box::new(assets::Asset {}))?;

		let parent = layout.root_widget;

		let res = wgui::parser::parse(
			&mut layout,
			parent,
			std::fs::read_to_string(XML_PATH).unwrap().as_str(),
		)?;

		Ok(Self { layout })
	}
}

impl Testbed for TestbedDashboard {
	fn update(&mut self, width: f32, height: f32, timestep_alpha: f32) -> anyhow::Result<()> {
		self
			.layout
			.update(Vec2::new(width, height), timestep_alpha)?;
		Ok(())
	}

	fn layout(&mut self) -> &mut Layout {
		&mut self.layout
	}
}
