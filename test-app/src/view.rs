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

		// let button_rect = Aabb2::around_point(Vec2::new(0.0, 100.0), Vec2::splat(30.0)).translate(Vec2::splat(0.5));

		// // Viewspace Button
		// if let Some(cursor_pos) = self.input.cursor_pos_view
		// 	&& button_rect.contains_point(cursor_pos)
		// {
		// 	painter.rounded_rect(button_rect, 10.0, [0.1; 3]);
		// }

		// painter.set_line_width(1.0);
		// painter.rounded_rect_outline(button_rect, 10.0, [1.0, 1.0, 1.0]);
		// painter.rounded_rect_outline(button_rect.translate(Vec2::new(0.25, -62.0)), 10.0, [1.0, 1.0, 1.0]);
		// painter.rounded_rect_outline(button_rect.translate(Vec2::new(0.5, -124.0)), 10.0, [1.0, 1.0, 1.0]);
		// painter.rounded_rect_outline(button_rect.translate(Vec2::new(0.75, -186.0)), 10.0, [1.0, 1.0, 1.0]);
		// painter.rounded_rect_outline(button_rect.translate(Vec2::new(1.0, -248.0)), 10.0, [1.0, 1.0, 1.0]);

		// painter.set_line_width(1.0);
		// painter.rect_outline(view_bounds.shrink(Vec2::splat(1.0)), [1.0, 0.5, 1.0]);

		// // Menu
		// let menu_bar_min = view_bounds.min_max_corner() - Vec2::from_y(24.0);
		// let menu_bounds = Aabb2::new(menu_bar_min, view_bounds.max);

		// painter.rect(menu_bounds, [0.02; 3]);

		self.ui.run(view_bounds.shrink(Vec2::splat(20.0)), painter, |ui| {
			ui.dummy();
			ui.dummy();

			let wdg = ui.add_widget(ui::BoxLayout{});
			ui.add_widget_to((), wdg);
			
			let wdg2 = ui.add_widget_to(ui::BoxLayout{}, wdg);
			ui.add_widget_to((), wdg2);

			ui.mutate_widget_constraints(wdg, |lc| {
				// lc.min_height.set(100.0);
				lc.margin.set_vertical(10.0);
				lc.margin.set_horizontal(20.0);
			});

			ui.dummy();


			let wdg3 = ui.add_widget(());
			ui.mutate_widget_constraints(wdg3, |lc| {
				lc.min_width.set(50.0);
				lc.preferred_width.set(100.0);
				lc.preferred_height.set(75.0);
			});

			let wdg3 = ui.add_widget(());
			ui.mutate_widget_constraints(wdg3, |lc| {
				lc.min_width.set(25.0);
				lc.preferred_width.set(200.0);
				lc.preferred_height.set(100.0);
			});
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
	pub cursor_pos_board: Option<Vec2>,

	pub events_received_this_frame: bool,
}

impl Input {
	pub fn prepare_frame(&mut self) {
		self.cursor_pos_view = None;
		self.cursor_pos_board = None;
		self.events_received_this_frame = false;
	}

	pub fn process_events(&mut self, viewport: &Viewport) {
		if let Some(raw_pos) = self.raw_cursor_pos {
			let view_pos = viewport.physical_to_view() * raw_pos;
			let board_pos = viewport.view_to_board() * view_pos;

			self.cursor_pos_view = Some(view_pos);
			self.cursor_pos_board = Some(board_pos);
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
	pub centre: Vec2,
	pub zoom: f32,
}

impl Viewport {
	pub fn new() -> Viewport {
		Viewport {
			size: Vec2::zero(),
			centre: Vec2::new(100.0, -50.0),
			zoom: 1.0,
		}
	}

	pub fn board_to_view(&self) -> Mat2x3 {
		let scale = self.zoom.recip();
		Mat2x3::scale_translate(Vec2::splat(scale), -self.centre * scale)
	}

	pub fn view_to_board(&self) -> Mat2x3 {
		Mat2x3::scale_translate(Vec2::splat(self.zoom), self.centre)
	}

	pub fn view_to_clip(&self) -> Mat2x3 {
		// ?????
		Mat2x3::scale_translate(Vec2::splat(2.0) / self.size, -Vec2::splat(0.0) / self.size)
	}

	pub fn physical_to_view(&self) -> Mat2x3 {
		let flip = Vec2::new(1.0, -1.0);
		Mat2x3::scale_translate(flip, -flip * self.size / 2.0)
	}

	pub fn view_bounds(&self) -> Aabb2 {
		Aabb2::around_point(Vec2::zero(), self.size / 2.0)
	}
}