#![feature(let_chains)]

use winit::{
	event::{Event, WindowEvent, StartCause},
	event_loop::{EventLoop, ControlFlow},
	window::{WindowBuilder, WindowLevel},
};

use std::sync::Arc;

use common::*;


pub mod renderer;
pub mod painter;
pub mod app;
pub mod view;
pub mod util;

pub mod prelude {
	pub use common::*;

	pub use super::{painter, renderer, app, view};
	pub use super::util::*;
	
	pub use painter::Painter;
	pub use app::{ItemKey, ImageKey};

	pub use smallvec::SmallVec;
	pub use slotmap::{SlotMap, SecondaryMap};

	pub use bitflags::bitflags;

	pub use std::collections::{VecDeque, HashMap, HashSet};
	pub use std::cell::{Cell, RefCell};
}



#[tokio::main(worker_threads=4)]
async fn main() -> anyhow::Result<()> {
	std::env::set_var("RUST_BACKTRACE", "1");

	env_logger::init();

	let event_loop = EventLoop::new()?;

	let window_builder = WindowBuilder::new()
		.with_title("refboard")
		.with_resizable(true)
		// .with_transparent(true) // Doesn't work
		// .with_decorations(false)
		.with_window_level(WindowLevel::AlwaysOnTop)
		.with_visible(false);

	// #[cfg(windows)] {
	// 	use winit::platform::windows::WindowBuilderExtWindows;
	// 	window_builder = window_builder.with_no_redirection_bitmap(true);
	// }

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

	let mut app = app::App::default();
	let mut view = view::View::new();
	view.set_size({
		let physical_size = window.inner_size().cast();
		Vec2i::new(physical_size.width, physical_size.height)
	});

	event_loop.set_control_flow(ControlFlow::Wait);

	event_loop.run(move |event, target| {
		match event {
			// Initial present/show window
			Event::NewEvents(StartCause::Init) => {
				renderer.prepare(&painter, &view.viewport, &mut *view.ui.text_state.borrow_mut());

				window.pre_present_notify();
				renderer.present();

				// Only set visible now to avoid flashing on startup
				window.set_visible(true);
			}

			Event::WindowEvent { window_id: _, event } => {
				match event {
					WindowEvent::RedrawRequested => {
						view.update_input();

						painter.clear();
						view.paint(&mut painter, &app);
						renderer.prepare(&painter, &view.viewport, &mut *view.ui.text_state.borrow_mut());

						window.pre_present_notify();
						renderer.present();

						app.apply_changes();

						view.prepare_next_frame();
					}

					WindowEvent::Resized(new_physical_size) => {
						renderer.resize(new_physical_size.width, new_physical_size.height);
						view.set_size(Vec2i::new(new_physical_size.width as i32, new_physical_size.height as i32));
					}

					// TODO(pat.m): theme change
					// TODO(pat.m): dpi change

					WindowEvent::CloseRequested => {
						// TODO(pat.m): this should defer to the view
						target.exit();
					}

					WindowEvent::MouseInput{..}
						| WindowEvent::CursorEntered{..}
						| WindowEvent::CursorLeft{..}
						| WindowEvent::CursorMoved{..}
						| WindowEvent::MouseWheel{..}
						| WindowEvent::KeyboardInput{..}
					=> {
						view.input.send_event(event);
					}

					_ => {}
				}
			}

			Event::AboutToWait => {
				if app.hack_changed.get() {
					app.hack_changed.set(false);
					window.request_redraw();
				}

				if view.should_redraw() {
					window.request_redraw();
				}
			}

			_ => {}
		}
	})
	.map_err(Into::into)
}

