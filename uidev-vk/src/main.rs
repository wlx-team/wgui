use glam::{Vec2, vec2};
use std::sync::Arc;
use testbed::Testbed;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use vulkan::init_window;
use wgui::{
	event::{MouseDownEvent, MouseMotionEvent, MouseUpEvent},
	gfx::WGfx,
	renderers::{
		rect::{RectPipeline, RectRenderer},
		text::{
			text_atlas::{TextAtlas, TextPipeline},
			text_renderer::TextRenderer,
		},
		viewport::Viewport,
	},
	vulkano::{
		Validated, VulkanError,
		command_buffer::CommandBufferUsage,
		format::Format,
		image::{ImageUsage, view::ImageView},
		swapchain::{
			Surface, SurfaceInfo, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
			acquire_next_image,
		},
		sync::GpuFuture,
	},
};
use winit::{
	event::{Event, WindowEvent},
	event_loop::ControlFlow,
};

mod profiler;
mod testbed;
mod vulkan;

pub struct Goodies {
	viewport: Viewport,
	text_renderer: TextRenderer,
	text_atlas: TextAtlas,
	rect_renderer: RectRenderer,
}

fn init_logging() {
	tracing_subscriber::registry()
		.with(
			tracing_subscriber::fmt::layer()
				.pretty()
				.with_writer(std::io::stderr),
		)
		.with(
			/* read RUST_LOG env var */
			EnvFilter::builder()
				.with_default_directive(LevelFilter::DEBUG.into())
				.from_env_lossy(),
		)
		.init();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	init_logging();

	let (gfx, event_loop, window, surface) = init_window()?;
	let inner_size = window.inner_size();
	let mut swapchain_size = [inner_size.width, inner_size.height];

	let native_format = gfx
		.device
		.physical_device()
		.surface_formats(&surface, SurfaceInfo::default())
		.unwrap()[0] // want panic
		.0;
	log::info!("Using surface format: {native_format:?}");

	let mut swapchain_create_info =
		swapchain_create_info(&gfx, native_format, surface.clone(), swapchain_size);

	let (mut swapchain, mut images) = {
		let (swapchain, images) = Swapchain::new(
			gfx.device.clone(),
			surface.clone(),
			swapchain_create_info.clone(),
		)?;

		let image_views = images
			.into_iter()
			.map(|image| ImageView::new_default(image).unwrap())
			.collect::<Vec<_>>();

		(swapchain, image_views)
	};

	let mut recreate = false;
	let mut last_draw = std::time::Instant::now();

	let rect_pipeline = RectPipeline::new(gfx.clone(), native_format)?;
	let text_pipeline = TextPipeline::new(gfx.clone(), native_format)?;
	let mut atlas = TextAtlas::new(text_pipeline.clone())?;

	let mut goodies = Goodies {
		viewport: Viewport::new(gfx.clone())?,
		text_renderer: TextRenderer::new(&mut atlas)?,
		text_atlas: atlas,
		rect_renderer: RectRenderer::new(rect_pipeline)?,
	};

	let mut testbed = Testbed::new()?;
	let mut mouse = Vec2::ZERO;

	goodies.viewport.update(swapchain_size)?;
	println!("new swapchain_size: {swapchain_size:?}");

	let mut profiler = profiler::Profiler::new(500);

	#[allow(deprecated)]
	event_loop.run(move |event, elwt| {
		elwt.set_control_flow(ControlFlow::Poll);

		match event {
			Event::WindowEvent {
				event: WindowEvent::MouseInput { state, button, .. },
				..
			} => {
				if matches!(button, winit::event::MouseButton::Left) {
					if matches!(state, winit::event::ElementState::Pressed) {
						testbed
							.layout
							.push_event(&wgui::event::Event::MouseDown(MouseDownEvent {
								pos: mouse,
							}))
							.unwrap();
					} else {
						testbed
							.layout
							.push_event(&wgui::event::Event::MouseUp(MouseUpEvent { pos: mouse }))
							.unwrap();
					}
				}
			}
			Event::WindowEvent {
				event: WindowEvent::CursorMoved { position, .. },
				..
			} => {
				mouse = vec2(position.x as _, position.y as _);
				testbed
					.layout
					.push_event(&wgui::event::Event::MouseMotion(MouseMotionEvent {
						pos: mouse,
					}))
					.unwrap();
			}
			Event::WindowEvent {
				event: WindowEvent::CloseRequested,
				..
			} => {
				elwt.exit();
			}
			Event::WindowEvent {
				event: WindowEvent::Resized(_),
				..
			} => {
				recreate = true;
			}
			Event::WindowEvent {
				event: WindowEvent::RedrawRequested,
				..
			} => {
				profiler.start();

				if recreate {
					let inner_size = window.inner_size();
					swapchain_size = [inner_size.width, inner_size.height];

					swapchain_create_info.image_extent = swapchain_size;

					(swapchain, images) = {
						let (swapchain, images) = swapchain.recreate(swapchain_create_info.clone()).unwrap();

						let image_views = images
							.into_iter()
							.map(|image| ImageView::new_default(image).unwrap())
							.collect::<Vec<_>>();

						(swapchain, image_views)
					};

					goodies.viewport.update(swapchain_size).unwrap();
					println!("new swapchain_size: {swapchain_size:?}");
					recreate = false;
					window.request_redraw();
				}

				{
					let (image_index, _, acquire_future) =
						match acquire_next_image(swapchain.clone(), None).map_err(Validated::unwrap) {
							Ok(r) => r,
							Err(VulkanError::OutOfDate) => {
								recreate = true;
								return;
							}
							Err(e) => panic!("failed to acquire next image: {e}"),
						};

					let tgt = images[image_index as usize].clone();

					last_draw = std::time::Instant::now();

					testbed
						.update(swapchain_size[0] as _, swapchain_size[1] as _)
						.unwrap();

					let mut cmd_buf = gfx
						.create_gfx_command_buffer(CommandBufferUsage::OneTimeSubmit)
						.unwrap();
					cmd_buf.begin_rendering(tgt).unwrap();

					testbed.draw(&mut cmd_buf, &mut goodies).unwrap();

					cmd_buf.end_rendering().unwrap();

					let cmd_buf = cmd_buf.build().unwrap();

					acquire_future
						.then_execute(gfx.queue_gfx.clone(), cmd_buf)
						.unwrap()
						.then_swapchain_present(
							gfx.queue_gfx.clone(),
							SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_index),
						)
						.then_signal_fence_and_flush()
						.unwrap()
						.wait(None)
						.unwrap();
				}

				profiler.end();
			}
			Event::AboutToWait => {
				if last_draw.elapsed().as_millis() > 16 {
					window.request_redraw();
				}
			}
			_ => (),
		}
	})?;

	Ok(())
}

fn swapchain_create_info(
	graphics: &WGfx,
	format: Format,
	surface: Arc<Surface>,
	extent: [u32; 2],
) -> SwapchainCreateInfo {
	let surface_capabilities = graphics
		.device
		.physical_device()
		.surface_capabilities(&surface, SurfaceInfo::default())
		.unwrap(); // want panic

	SwapchainCreateInfo {
		min_image_count: surface_capabilities.min_image_count.max(2),
		image_format: format,
		image_extent: extent,
		image_usage: ImageUsage::COLOR_ATTACHMENT,
		composite_alpha: surface_capabilities
			.supported_composite_alpha
			.into_iter()
			.next()
			.unwrap(), // want panic
		..Default::default()
	}
}
