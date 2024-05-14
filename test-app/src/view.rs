use crate::prelude::*;

mod ui;

pub struct View {
	// item view
	// gizmo overlay
	// menus

	pub viewport: Viewport,
	pub input: Input,
	pub ui: ui::System,

	should_redraw: bool,
	time: f32,
}

impl View {
	pub fn new() -> View {
		let input = Input::default();
		let ui = ui::System::new();

		View {
			viewport: Viewport::default(),
			input,
			ui,
			time: 0.0,
			should_redraw: true,
		}
	}

	pub fn set_size(&mut self, new_size: Vec2i) {
		if self.viewport.size.to_vec2i() != new_size {
			self.should_redraw = true;
			self.viewport.size = new_size.to_vec2();
		}
	}

	pub fn prepare_frame(&mut self) {
		self.should_redraw = false;
		self.input.prepare_frame();
	}

	pub fn update_input(&mut self) {
		self.input.process_events(&self.viewport);
	}

	pub fn paint(&mut self, painter: &mut Painter, _app: &app::App) {
		let view_bounds = self.viewport.view_bounds();

		self.ui.run(view_bounds, painter, &self.input, |ui| {
			let frame = ui.add_widget(ui::FrameWidget::horizontal());

			if frame.is_hovered() {
				frame.widget(|frame| {
					frame.background_color = Color::grey_a(0.5, 0.1);
				});
			}

			let add_widget = || {
				let frame = ui.add_widget(ui::FrameWidget::horizontal().with_color(Color::green()))
					.set_constraints(|c| {
						c.set_size((50.0, 50.0));
					});

				if frame.is_hovered() {
					frame.widget(|frame| {
						frame.background_color = Color::light_green();
					});
				}

				frame
			};

			ui.push_layout(frame);
			add_widget().set_constraints(|c| c.set_size((20.0, 20.0)));
			add_widget().set_constraints(|c| c.set_size((50.0, 50.0)));
			add_widget().set_constraints(|c| c.set_size((100.0, 50.0)));
			add_widget().set_constraints(|c| c.set_size((50.0, 100.0)));
			ui.pop_layout();
		});

		// Cursor
		if let Some(cursor_pos) = self.input.cursor_pos_view {
			painter.circle(cursor_pos, 3.0, [1.0, 0.0, 0.0]);
		}

		self.time += 1.0/60.0;
	}

	pub fn should_redraw(&self) -> bool {
		self.should_redraw || self.input.events_received_this_frame
	}
}




use winit::event::WindowEvent;
use winit::dpi::PhysicalPosition;

#[derive(Default)]
pub struct Input {
	raw_cursor_pos: Option<Vec2>,

	pub cursor_pos_view: Option<Vec2>,

	pub events_received_this_frame: bool,
}

impl Input {
	pub fn prepare_frame(&mut self) {
		self.cursor_pos_view = None;
		self.events_received_this_frame = false;
	}

	pub fn process_events(&mut self, viewport: &Viewport) {
		if let Some(raw_pos) = self.raw_cursor_pos {
			let view_pos = viewport.physical_to_view() * raw_pos;
			self.cursor_pos_view = Some(view_pos);
		}
	}

	pub fn send_event(&mut self, event: WindowEvent) {
		self.events_received_this_frame = false;

		match event {
			WindowEvent::CursorMoved { position, .. } => {
				let PhysicalPosition {x, y} = position.cast();
				self.raw_cursor_pos = Some(Vec2::new(x, y));
			}


			WindowEvent::CursorLeft { .. } => {
				self.raw_cursor_pos = None;
			}

			_ => {}
		}
	}
}



#[derive(Debug, Default)]
pub struct Viewport {
	pub size: Vec2,
}

impl Viewport {
	pub fn new() -> Viewport {
		Viewport {
			size: Vec2::zero(),
		}
	}

	pub fn view_to_clip(&self) -> Mat2x3 {
		Mat2x3::scale_translate(Vec2::new(2.0, -2.0) / self.size, Vec2::new(-1.0, 1.0))
	}

	pub fn physical_to_view(&self) -> Mat2x3 {
		Mat2x3::identity()
	}

	pub fn view_bounds(&self) -> Aabb2 {
		Aabb2::new(Vec2::zero(), self.size)
	}
}