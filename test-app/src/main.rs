#![feature(let_chains)]

use winit::{
	application::ApplicationHandler,
	event::{Event, WindowEvent, StartCause},
	event_loop::{ActiveEventLoop, EventLoop, ControlFlow},
	window::{Window, WindowId, /*WindowLevel*/},
};

use std::sync::Arc;

use common::*;


pub mod ui;

pub mod renderer;
pub mod painter;
pub mod view;
pub mod util;

pub mod prelude {
	pub use common::*;

	pub use super::{painter, renderer, view, ui};
	pub use super::util::*;

	pub use ui::{StatefulWidget};
	
	pub use painter::Painter;

	pub use smallvec::SmallVec;
	pub use slotmap::{SlotMap, SecondaryMap};

	pub use bitflags::bitflags;

	pub use std::collections::{VecDeque, HashMap, HashSet};
	pub use std::cell::{Cell, RefCell};
	pub use std::num::Wrapping;
	
	pub use tracing;
	#[doc(hidden)]
	pub use tracing::instrument;
}



fn main() -> anyhow::Result<()> {
	std::env::set_var("RUST_BACKTRACE", "1");

	env_logger::init();

	#[cfg(feature="tracy")]
	init_tracy();

	let event_loop = EventLoop::new()?;
	event_loop.set_control_flow(ControlFlow::Wait);
	event_loop.run_app(&mut ApplicationHost::new())
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



struct ApplicationHost {
	ui_system: ui::System,
	painter: painter::Painter,
	view: view::View,

	app_window: Option<ApplicationWindow>,
}

impl ApplicationHost {
	pub fn new() -> Self {
		let painter = painter::Painter::new();
		let ui_system = ui::System::new();
		let view = view::View::new();

		ApplicationHost {
			painter,
			ui_system,
			view,

			app_window: None,
		}
	}

	fn redraw(&mut self) {
		let Some(ApplicationWindow{window, renderer}) = self.app_window.as_mut() else {
			return
		};

		self.painter.clear();
		self.ui_system.run(&mut self.painter, |ui| {
			self.view.build(ui);
		});
		
		renderer.prepare(&self.painter, &self.ui_system.viewport, &mut *self.ui_system.text_atlas.borrow_mut());

		window.pre_present_notify();
		renderer.present();

		self.ui_system.prepare_next_frame();
	}
}


impl ApplicationHandler for ApplicationHost {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window_attributes = Window::default_attributes()
			.with_title("ui fuckery")
			.with_resizable(true)
			.with_transparent(true)
			.with_decorations(false)
			.with_visible(false);

		let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
		let renderer = pollster::block_on(renderer::Renderer::start(window.clone())).unwrap();

		self.ui_system.set_size({
			let physical_size = window.inner_size().cast();
			Vec2i::new(physical_size.width, physical_size.height)
		});

		self.app_window = Some(ApplicationWindow {
			window: window.clone(),
			renderer,
		});

		self.redraw();

		window.set_visible(true);
	}

	fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
		match event {
			WindowEvent::RedrawRequested => {
				self.redraw();
			}

			WindowEvent::Resized(new_physical_size) => {
				use winit::dpi::PhysicalSize;

				let Some(ApplicationWindow{window, renderer}) = self.app_window.as_mut() else {
					return
				};

				renderer.resize(new_physical_size.width, new_physical_size.height);
				self.ui_system.set_size(Vec2i::new(new_physical_size.width as i32, new_physical_size.height as i32));

				let Vec2i{x, y} = self.ui_system.min_size;
				let new_min_size = PhysicalSize::new(x as u32, y as u32);
				window.set_min_inner_size(Some(new_min_size));
			}

			// TODO(pat.m): theme change
			// TODO(pat.m): dpi change

			WindowEvent::CloseRequested => {
				// TODO(pat.m): this should defer to the view
				event_loop.exit();
			}

			WindowEvent::MouseInput{..}
				| WindowEvent::CursorEntered{..}
				| WindowEvent::CursorLeft{..}
				| WindowEvent::CursorMoved{..}
				| WindowEvent::MouseWheel{..}
				| WindowEvent::KeyboardInput{..}
			=> {
				let Some(ApplicationWindow{window, ..}) = self.app_window.as_mut() else {
					return
				};

				match self.ui_system.input.send_event(event) {
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

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
		use winit::dpi::PhysicalSize;

		if self.view.wants_quit {
			event_loop.exit();
		}

		let Some(ApplicationWindow{window, ..}) = self.app_window.as_mut() else {
			return
		};

		if self.ui_system.should_redraw() {
			window.request_redraw();
		}

		// If the minsize of the window has changed make sure we update it
		let Vec2i{x, y} = self.ui_system.min_size;
		let new_min_size = PhysicalSize::new(x as u32, y as u32);
		let current_inner_size = window.inner_size();

		if current_inner_size.width < new_min_size.width || current_inner_size.height < new_min_size.height {
			window.set_min_inner_size(Some(new_min_size));

			let new_current_size = PhysicalSize::new((x as u32).max(new_min_size.width), (y as u32).max(new_min_size.height));
			let _ = window.request_inner_size(new_current_size);
		}
    }
}


struct ApplicationWindow {
	window: Arc<Window>,
	renderer: renderer::Renderer,
}