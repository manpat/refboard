use crate::prelude::*;

pub struct View {
	// item view
	// gizmo overlay
	// menus

	pub wants_quit: bool,
}

impl View {
	pub fn new() -> View {
		View {
			wants_quit: false,
		}
	}

	pub fn build(&mut self, ui: &ui::Ui<'_>, _app: &app::App) {
		ui.with_vertical_frame(|| {
			self.draw_menu_bar(ui);

			ui.with_vertical_layout(|| {
				self.draw_content(ui);
			})
			.with_constraints(|c| {
				c.set_size_policy(ui::SizingBehaviour::CAN_GROW);
				c.min_height.set(2.0);
			});

			self.draw_status_bar(ui);
		})
		.with_style(|style| {
			style.set_fill(ui::WidgetColorRole::SurfaceContainerLowest);
			style.rounding = Some(painter::BorderRadii::new(8.0));
		})
		.with_constraints(|c| {
			c.set_size_policy(ui::SizingBehaviour::CAN_GROW);
			c.padding.set(0.0);
			c.margin.set(0.0);
		});
	}

	fn draw_menu_bar(&mut self, ui: &ui::Ui<'_>) {
		ui.with_horizontal_frame(|| {
			ui.button("File").with_constraints(|c| {c.margin.set_horizontal(2.0); c.padding.set_vertical(4.0); });
			ui.button("Foo Bar").with_constraints(|c| {c.margin.set_horizontal(2.0); c.padding.set_vertical(4.0); });
			ui.button("Weh").with_constraints(|c| {c.margin.set_horizontal(2.0); c.padding.set_vertical(4.0); });

			ui.spring(ui::Axis::Horizontal);

			let close_button = ui.button("x")
				.with_constraints(|c| c.set_size((24.0, 24.0)))
				.with_style(|s| s.set_fill(ui::WidgetColorRole::ErrorContainer));

			if close_button.is_clicked() {
				self.wants_quit = true;
			}
		})
		.with_style(|style| {
			style.rounding = Some(painter::BorderRadii {
				top_left: 8.0,
				top_right: 8.0,
				bottom_left: 0.0,
				bottom_right: 0.0,
			});
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

	fn draw_status_bar(&mut self, ui: &ui::Ui<'_>) {
		ui.with_horizontal_frame(|| {
			ui.spring(ui::Axis::Horizontal);

			ui.text("Resize")
				.with_input_behaviour(ui::InputBehaviour::WINDOW_DRAG_RESIZE_ZONE)
				.with_style(|s| s.set_outline(ui::WidgetColorRole::Outline))
				.with_constraints(|c| {
					c.padding.set(8.0);
					c.margin.set(0.0);
				});
		})
		.with_style(|style| {
			style.rounding = Some(painter::BorderRadii {
				top_left: 0.0,
				top_right: 0.0,
				bottom_left: 8.0,
				bottom_right: 8.0,
			});
		})
		.with_constraints(|c| {
			c.horizontal_size_policy.set(ui::SizingBehaviour::CAN_GROW);
			c.padding.set(0.0);
		});
	}

	fn draw_content(&mut self, ui: &ui::Ui<'_>) {
		ui.with_horizontal_layout(|| {
			ui.button("Foo");
			ui.button("Foo").style().set_fill(ui::WidgetColorRole::Primary);
			ui.button("Foo").style().set_fill(ui::WidgetColorRole::Secondary);
			ui.button("Foo").style().set_fill(ui::WidgetColorRole::Tertiary);

			ui.button("Foo").style().set_fill(ui::WidgetColorRole::PrimaryContainer);
			ui.button("Foo").style().set_fill(ui::WidgetColorRole::SecondaryContainer);
			ui.button("Foo").style().set_fill(ui::WidgetColorRole::TertiaryContainer);

			ui.button("Foo").style().set_outline(ui::WidgetColorRole::Outline);
			ui.button("Foo").style().set_outline(ui::WidgetColorRole::OutlineVariant);
		});

		ui.with_horizontal_layout(|| {
			ui.text("Foo").with_constraints(|c| c.padding.set(4.0));
			ui.text("Foo").with_constraints(|c| c.padding.set(4.0)).style().set_fill(ui::WidgetColorRole::Primary);
			ui.text("Foo").with_constraints(|c| c.padding.set(4.0)).style().set_fill(ui::WidgetColorRole::Secondary);
			ui.text("Foo").with_constraints(|c| c.padding.set(4.0)).style().set_fill(ui::WidgetColorRole::Tertiary);

			ui.text("Foo").with_constraints(|c| c.padding.set(4.0)).style().set_fill(ui::WidgetColorRole::PrimaryContainer);
			ui.text("Foo").with_constraints(|c| c.padding.set(4.0)).style().set_fill(ui::WidgetColorRole::SecondaryContainer);
			ui.text("Foo").with_constraints(|c| c.padding.set(4.0)).style().set_fill(ui::WidgetColorRole::TertiaryContainer);

			ui.text("Foo").with_constraints(|c| c.padding.set(4.0)).style().set_fill(ui::WidgetColorRole::Surface);
			ui.text("Foo").with_constraints(|c| c.padding.set(4.0)).style().set_fill(ui::WidgetColorRole::SurfaceContainerHighest);
			ui.text("Foo").with_constraints(|c| c.padding.set(4.0)).style().set_fill(ui::WidgetColorRole::SurfaceContainerHigh);
			ui.text("Foo").with_constraints(|c| c.padding.set(4.0)).style().set_fill(ui::WidgetColorRole::SurfaceContainer);
			ui.text("Foo").with_constraints(|c| c.padding.set(4.0)).style().set_fill(ui::WidgetColorRole::SurfaceContainerLow);
			ui.text("Foo").with_constraints(|c| c.padding.set(4.0)).style().set_fill(ui::WidgetColorRole::SurfaceContainerLowest);

			ui.text("Foo").with_constraints(|c| c.padding.set(4.0)).style().set_outline(ui::WidgetColorRole::Outline);
			ui.text("Foo").with_constraints(|c| c.padding.set(4.0)).style().set_outline(ui::WidgetColorRole::OutlineVariant);
		});

		ui.with_parent(ui.button("_____________"), || {
			ui.text("I'm on top")
				.with_input_behaviour(ui::InputBehaviour::TRANSPARENT);
		})
	}
}




