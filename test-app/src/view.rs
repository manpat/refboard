use crate::prelude::*;

pub struct View {
	// item view
	// gizmo overlay
	// menus
}

impl View {
	pub fn new() -> View {
		View {}
	}

	pub fn build(&mut self, ui: &ui::Ui<'_>, app: &app::App) {
		ui.with_vertical_frame(|| {
			self.draw_menu_bar(ui);

			ui.spring(ui::Axis::Vertical);

			ui.with_horizontal_frame(|| {
				ui.text("Resize")
					.with_input_behaviour(ui::InputBehaviour::WINDOW_DRAG_RESIZE_ZONE);

				ui.spring(ui::Axis::Horizontal);

				ui.text("Resize")
					.with_input_behaviour(ui::InputBehaviour::WINDOW_DRAG_RESIZE_ZONE);
			})
			.with_constraints(|c| c.horizontal_size_policy.set(ui::SizingBehaviour::CAN_GROW));
		})
		.with_widget(|w, _| w.background_color = Color::grey(0.02))
		.with_constraints(|c| {
			c.set_size_policy(ui::SizingBehaviour::CAN_GROW);
			c.padding.set(0.0);
			c.margin.set(0.0);
		});
	}

	fn draw_menu_bar(&mut self, ui: &ui::Ui<'_>) {
		ui.with_horizontal_frame(|| {
			ui.text("File").with_constraints(|c| c.margin.set_horizontal(4.0));
			ui.text("Foo Bar").with_constraints(|c| c.margin.set_horizontal(4.0));
			ui.text("Weh").with_constraints(|c| c.margin.set_horizontal(4.0));

			ui.spring(ui::Axis::Horizontal);

			ui.button()
				.with_constraints(|c| c.set_size((32.0, 32.0)));
		})
		.with_input_behaviour(ui::InputBehaviour::WINDOW_DRAG_ZONE)
		.with_constraints(|c| {
			c.horizontal_size_policy.set(ui::SizingBehaviour::CAN_GROW);
			c.content_alignment.set(ui::Align::Middle);
			c.margin.set(0.0);
			c.padding.set_vertical(0.0);
			c.padding.right.set(0.0);
		});
	}
}




