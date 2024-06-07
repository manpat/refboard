#![feature(let_chains)]

use winit::{
	event::{Event, WindowEvent, StartCause},
	event_loop::{EventLoop, ControlFlow},
	window::{Window, /*WindowLevel*/},
};

use std::sync::Arc;

use common::*;


pub mod ui;

pub mod renderer;
pub mod painter;
pub mod app;
pub mod view;
pub mod util;

pub mod prelude {
	pub use common::*;

	pub use super::{painter, renderer, app, view, ui};
	pub use super::util::*;
	
	pub use painter::Painter;
	pub use app::{ItemKey, ImageKey};

	pub use smallvec::SmallVec;
	pub use slotmap::{SlotMap, SecondaryMap};

	pub use bitflags::bitflags;

	pub use std::collections::{VecDeque, HashMap, HashSet};
	pub use std::cell::{Cell, RefCell};
	
	pub use tracing;
	#[doc(hidden)]
	pub use tracing::instrument;
}



#[tokio::main(worker_threads=4)]
async fn main() -> anyhow::Result<()> {
	std::env::set_var("RUST_BACKTRACE", "1");

	env_logger::init();

	#[cfg(feature="tracy")]
	init_tracy();

	let event_loop = EventLoop::new()?;

	let window_attributes = Window::default_attributes()
		.with_title("ui fuckery")
		.with_resizable(true)
		.with_transparent(true)
		.with_decorations(false)
		.with_visible(false);

	// TODO(pat.m): rewrite all this shit again for winit 0.30
	#[allow(deprecated)]
	let window = Arc::new(event_loop.create_window(window_attributes)?);

	let mut renderer = renderer::Renderer::start(window.clone()).await?;
	let mut painter = painter::Painter::new();
	let mut ui_system = ui::System::new();
	ui_system.set_size({
		let physical_size = window.inner_size().cast();
		Vec2i::new(physical_size.width, physical_size.height)
	});

	let mut app = app::App::default();
	let mut view = view::View::new();

	event_loop.set_control_flow(ControlFlow::Wait);

	// TODO(pat.m): rewrite all this shit again for winit 0.30
	#[allow(deprecated)]
	event_loop.run(move |event, target| {
		match event {
			// Initial present/show window
			Event::NewEvents(StartCause::Init) => {
				ui_system.run(&mut painter, |ui| {
					view.build(ui, &app);
				});

				renderer.prepare(&painter, &ui_system.viewport, &mut *ui_system.text_atlas.borrow_mut());

				window.pre_present_notify();
				renderer.present();

				// Only set visible now to avoid flashing on startup
				window.set_visible(true);
			}

			Event::WindowEvent { window_id: _, event } => {
				match event {
					WindowEvent::RedrawRequested => {
						painter.clear();
						ui_system.run(&mut painter, |ui| {
							view.build(ui, &app);
						});
						
						renderer.prepare(&painter, &ui_system.viewport, &mut *ui_system.text_atlas.borrow_mut());

						window.pre_present_notify();
						renderer.present();

						app.apply_changes();

						ui_system.prepare_next_frame();
					}

					WindowEvent::Resized(new_physical_size) => {
						use winit::dpi::PhysicalSize;

						renderer.resize(new_physical_size.width, new_physical_size.height);
						ui_system.set_size(Vec2i::new(new_physical_size.width as i32, new_physical_size.height as i32));

						let Vec2i{x, y} = ui_system.min_size;
						let new_min_size = PhysicalSize::new(x as u32, y as u32);
						window.set_min_inner_size(Some(new_min_size));
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
						match ui_system.input.send_event(event) {
							ui::SendEventResponse::DragWindow => {
								if let Err(err) = window.drag_window() {
									println!("Window drag failed {err}");
								}
							}

							ui::SendEventResponse::DragResizeWindow(resize_direction) => {
								if let Err(err) = window.drag_resize_window(resize_direction) {
									println!("Window drag resize failed {err}");
								}
							}

							_ => {}
						}
					}

					_ => {}
				}
			}

			Event::AboutToWait => {
				use winit::dpi::PhysicalSize;

				if view.wants_quit {
					target.exit();
				}

				if app.hack_changed.get() {
					app.hack_changed.set(false);
					window.request_redraw();
				}

				if ui_system.should_redraw() {
					window.request_redraw();
				}

				// If the minsize of the window has changed make sure we update it
				let Vec2i{x, y} = ui_system.min_size;
				let new_min_size = PhysicalSize::new(x as u32, y as u32);
				let current_inner_size = window.inner_size();

				if current_inner_size.width < new_min_size.width || current_inner_size.height < new_min_size.height {
					window.set_min_inner_size(Some(new_min_size));
					let _ = window.request_inner_size(new_min_size);
				}
			}

			_ => {}
		}
	})
	.map_err(Into::into)
}



#[cfg(feature="tracy")]
fn init_tracy() {
    use tracing_subscriber::layer::SubscriberExt;

    let subscriber = tracing_subscriber::registry()
        .with(tracing_tracy::TracyLayer::new());

    tracing::subscriber::set_global_default(subscriber)
    	.expect("set up the subscriber");
    	
	println!("tracy init");
}