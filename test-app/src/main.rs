use winit::{
	event::{Event, WindowEvent, StartCause},
	event_loop::{EventLoop, ControlFlow},
	window::{WindowBuilder, WindowLevel},
};

use std::sync::Arc;

use common::*;


pub mod renderer;
pub mod painter;

pub mod prelude {
	pub use common::*;

	pub use super::painter::{self, Painter};
	pub use super::renderer;
}



#[tokio::main]
async fn main() -> anyhow::Result<()> {
	env_logger::init();

	// use winit::platform::windows::WindowBuilderExtWindows;

	let event_loop = EventLoop::new()?;
	let mut window_builder = WindowBuilder::new()
		.with_title("refboard")
		.with_resizable(true)
		// .with_transparent(true) // Doesn't work
		// .with_decorations(false)
		.with_window_level(WindowLevel::AlwaysOnTop)
		.with_visible(false);

	#[cfg(windows)] {
		use winit::platform::windows::WindowBuilderExtWindows;
		window_builder = window_builder.with_no_redirection_bitmap(true);
	}

	let window = Arc::new(window_builder.build(&event_loop)?);

	// #[cfg(windows)]
	// unsafe {
	// 	use raw_window_handle::{HasWindowHandle, RawWindowHandle};
	// 	use windows::{Win32::{UI::WindowsAndMessaging::*, Foundation::*}};

	// 	let RawWindowHandle::Win32(handle) = window.window_handle()?.as_raw() else { anyhow::bail!("Failed to get window handle"); };
	// 	let hwnd = HWND(handle.hwnd.get());

	// 	SetWindowLongPtrA(hwnd,
	// 		GWL_EXSTYLE,
	// 		GetWindowLongA(hwnd, GWL_EXSTYLE) as isize | WS_EX_LAYERED.0 as isize);

	// 	SetLayeredWindowAttributes(hwnd, COLORREF(0), ((255 * 30) / 100) as u8, LWA_ALPHA)?;
	// }

	println!("window created");

	let mut renderer = renderer::Renderer::start(window.clone()).await?;
	let mut painter = painter::Painter::new();

	let mut time = 0.0f32;

	event_loop.set_control_flow(ControlFlow::Wait);

	event_loop.run(move |event, target| {

		// Initial present/show window
		if let Event::NewEvents(StartCause::Init) = event {
			renderer.prepare(&painter);

			window.pre_present_notify();
			renderer.present();

			window.set_visible(true);
		}

		if let Event::AboutToWait = event {
			window.request_redraw();
		}

		if let Event::WindowEvent { window_id, event } = event {
			match event {
				WindowEvent::RedrawRequested => {
					use lyon::math::point;
					use lyon::path::Path;

					let mut builder = Path::builder();
					builder.begin(point(-7.0, 0.0));
					builder.line_to(point(-7.0, 7.0));
					builder.line_to(point(0.0, 7.0));
					builder.quadratic_bezier_to(point(7.0, 7.0), point(7.0, 0.0));
					builder.end(false);

					let path = builder.build();

					painter.clear();
					painter.fill_path(&path, [0.3, 0.2, 0.9]);
					painter.stroke_path(&path, [0.3, 1.0, 0.6]);

					painter.fill_circle(Vec2::zero(), 5.0, [1.0, 0.5, 1.0]);
					painter.circle(Vec2::from_y(time.sin()*5.0), 5.0, [0.3, 0.1, 0.35]);

					painter.rect(Aabb2::around_point(Vec2::zero(), Vec2::new(9.0, 9.0)), [1.0, 1.0, 1.0]);


					renderer.prepare(&painter);

					window.pre_present_notify();
					renderer.present();

					time += 1.0/60.0;
				}

				WindowEvent::Resized(new_size) => {
					renderer.resize(new_size.width, new_size.height);
					window.request_redraw();
				}

				WindowEvent::CloseRequested => {
					target.exit();
				}

				_ => {}
			}
		}
	})
	.map_err(Into::into)
}

