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
}

impl View {
	pub fn new() -> View {
		let input = Input::default();
		let ui = ui::System::new();

		View {
			viewport: Viewport::default(),
			input,
			ui,
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

	pub fn paint(&mut self, painter: &mut Painter, app: &app::App) {
		let view_bounds = self.viewport.view_bounds();

		self.ui.run(view_bounds, painter, &self.input, |ui| {
			ui.push_layout(
				ui.add_widget(ui::FrameWidget::vertical())
					.with_constraints(|c| c.set_size_policy(ui::SizingBehaviour::FLEXIBLE))
			);

			let add_widget = |idx| {
				let frame = ui.add_widget(ui::FrameWidget::horizontal().with_color(Color::green().with_alpha(0.2)));

				if frame.is_hovered() {
					frame.widget().background_color = Color::green();
				}

				if frame.is_clicked() {
					println!("CLICK! {idx}");
					app.dummy.set(app.dummy.get() + 1);
					app.hack_changed.set(true);
				}

				frame
			};

			ui.push_layout(ui.add_widget(ui::FrameWidget::horizontal()));
			for _ in 0..app.dummy.get() {
				add_widget(5).with_constraints(|c| c.set_size((50.0, 50.0)));
			}
			ui.pop_layout();

			let frame = ui.add_widget(ui::FrameWidget::horizontal())
				.with_constraints(|c| c.margin.set(8.0));

			if frame.is_hovered() {
				frame.widget().background_color = Color::grey_a(0.5, 0.1);
			}

			if frame.is_clicked() {
				app.dummy.set(app.dummy.get().saturating_sub(1));
				app.hack_changed.set(true);
			}

			ui.push_layout(frame);
			add_widget(0).constraints().set_size((20.0, 20.0));
			add_widget(1).constraints().set_size((50.0, 50.0));
			add_widget(2).constraints().set_size((100.0, 50.0));
			add_widget(3).constraints().set_size((50.0, 100.0));
			ui.pop_layout();


			ui.push_layout(ui.add_widget(ui::BoxLayout::horizontal()));
			{
				#[derive(Default, Debug)]
				struct Stateful;

				#[derive(Default)]
				struct StatefulState {
					active: bool,
					hovered: bool,
				}

				impl ui::Widget for Stateful {
					fn draw(&self, painter: &mut Painter, layout: &ui::Layout, state: &mut ui::StateBox) {
						let state = state.get::<StatefulState>();

						let color = match (state.active, state.hovered) {
							(true, false) => Color::green(),
							(true, true) => Color::light_green(),

							(false, false) => Color::red(),
							(false, true) => Color::light_red(),
						};

						ui::FrameWidget::horizontal()
							.with_color(color)
							.draw(painter, layout, &mut ui::StateBox::default());
					}
				}

				#[allow(non_local_definitions)]
				impl ui::WidgetRef<'_, Stateful> {
					fn is_active(&self) -> bool {
						self.state_as::<StatefulState>().active
					}
				}

				let widget = ui.add_widget(Stateful)
					.with_constraints(|c| c.set_size((32.0, 32.0)));

				widget.state_as::<StatefulState>().hovered = widget.is_hovered();

				if widget.is_clicked() {
					let mut state = widget.state_as::<StatefulState>();
					state.active = !state.active;
				}

				ui.add_widget(())
					.with_constraints(|c| match widget.is_active() {
						true => c.set_width(100.0),
						false => c.set_width(40.0),
					});
			}
			ui.pop_layout();


			ui.spring(ui::Axis::Vertical);

			ui.push_layout(ui.add_widget(ui::BoxLayout::horizontal()));

			if ui.button().is_clicked() {
				app.dummy.set(app.dummy.get() + 1);
				app.hack_changed.set(true);
			}

			if ui.button().is_clicked() {
				app.dummy.set(app.dummy.get().saturating_sub(1));
				app.hack_changed.set(true);
			}

			ui.pop_layout();

			ui.pop_layout();
		});

		// Cursor
		if let Some(cursor_pos) = self.input.cursor_pos_view {
			painter.circle(cursor_pos, 3.0, [1.0, 0.0, 0.0]);
		}
	}

	pub fn should_redraw(&self) -> bool {
		self.should_redraw || self.input.events_received_this_frame
	}
}




use winit::event::{WindowEvent, ElementState, MouseButton};
use winit::dpi::PhysicalPosition;

#[derive(Default)]
pub struct Input {
	raw_cursor_pos: Option<Vec2>,

	pub cursor_pos_view: Option<Vec2>,
	pub click_received: bool,

	pub events_received_this_frame: bool,
}

impl Input {
	pub fn prepare_frame(&mut self) {
		self.cursor_pos_view = None;
		self.click_received = false;
		self.events_received_this_frame = false;
	}

	pub fn process_events(&mut self, viewport: &Viewport) {
		self.cursor_pos_view = self.raw_cursor_pos
			.map(|raw_pos| viewport.physical_to_view() * raw_pos);
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

			WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
				// TODO(pat.m): better
				self.click_received = true;
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