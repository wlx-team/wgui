use sdl2::sys::{
	SDL_CreateRGBSurfaceWithFormatFrom, SDL_FillRect, SDL_FreeSurface, SDL_MapRGB,
	SDL_PixelFormatEnum, SDL_UpperBlit,
};
use testbed::Testbed;
use time::get_millis;
use tiny_skia::{Pixmap, Transform};
use tracing_subscriber::{
	EnvFilter, filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt,
};
use wgui::glam::Vec2;

pub mod testbed;
pub mod time;

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
				.with_default_directive(LevelFilter::INFO.into())
				.from_env_lossy(),
		)
		.init();
}

fn run() -> Result<(), String> {
	init_logging();

	let sdl_context = sdl2::init()?;
	let video = sdl_context.video()?;

	let width = 1280;
	let height = 720;

	let window = video
		.window("wgui", width, height)
		.resizable()
		.position_centered()
		.build()
		.map_err(|e| e.to_string())?;

	let mut event_pump = sdl_context.event_pump()?;
	let mut frames: u32 = 0;
	let mut last_frames_tick: u64 = 0;

	println!("Capping at 60 FPS");
	let mut rate = time::Rate::new(60);

	let mut testbed = Testbed::new().unwrap();

	'running: loop {
		rate.start();

		let (width, height) = window.size();

		use sdl2::event::Event as SDLEvent;
		use wgui::event as WguiEvent;

		for event in event_pump.poll_iter() {
			match event {
				SDLEvent::Quit { .. } => {
					break 'running;
				}
				SDLEvent::MouseButtonDown { x, y, .. } => {
					testbed
						.layout
						.push_event(&WguiEvent::Event::MouseDown(WguiEvent::MouseDownEvent {
							pos: Vec2::new(x as f32, y as f32),
						}))
						.unwrap();
				}
				SDLEvent::MouseButtonUp { x, y, .. } => {
					testbed
						.layout
						.push_event(&WguiEvent::Event::MouseUp(WguiEvent::MouseUpEvent {
							pos: Vec2::new(x as f32, y as f32),
						}))
						.unwrap();
				}
				SDLEvent::MouseMotion { x, y, .. } => {
					testbed
						.layout
						.push_event(&WguiEvent::Event::MouseMotion(
							WguiEvent::MouseMotionEvent {
								pos: Vec2::new(x as f32, y as f32),
							},
						))
						.unwrap();
				}
				_ => {}
			}
		}

		// this is customizable!
		let gui_scale = 1.0;
		let transform = Transform::from_scale(gui_scale, gui_scale);

		let mut pixmap = Pixmap::new(width, height).unwrap();

		testbed
			.update(pixmap.width() as f32, pixmap.height() as f32)
			.unwrap();
		testbed.draw(&mut pixmap, &transform).unwrap();

		let window_surface = window.surface(&event_pump)?;

		unsafe {
			// Clear window
			SDL_FillRect(
				window_surface.raw(),
				std::ptr::null(),
				SDL_MapRGB(window_surface.pixel_format().raw(), 255, 255, 255),
			);

			let pixels = pixmap.pixels().as_ptr();

			// Safe and fast, zero-copy blitting to SDL window
			let surf = SDL_CreateRGBSurfaceWithFormatFrom(
				pixels as *mut std::ffi::c_void,
				pixmap.width() as i32,
				pixmap.height() as i32,
				0,
				pixmap.width() as i32 * 4,
				SDL_PixelFormatEnum::SDL_PIXELFORMAT_RGBA32 as u32,
			);

			SDL_UpperBlit(
				surf,
				std::ptr::null(),
				window_surface.raw(),
				std::ptr::null_mut(),
			);

			SDL_FreeSurface(surf);
		}

		window_surface.update_window()?;

		frames += 1;
		let millis = get_millis();
		if last_frames_tick + 1000 < millis {
			last_frames_tick = millis;
			println!("{} FPS", frames);
			frames = 0;
		}

		rate.end();
	}

	Ok(())
}

fn main() {
	run().unwrap();
}
