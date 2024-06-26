use crate::prelude::*;
use std::num::Wrapping;

pub struct View {
	// item view
	// gizmo overlay
	// menus

	pub wants_quit: bool,
	pub frame_counter: Wrapping<u16>,

	pub button_clicks: u32,
	pub slider_value: f32,
	pub checkbox_value: bool,

	pub string_value: String,
}

impl View {
	pub fn new() -> View {
		View {
			wants_quit: false,
			frame_counter: Wrapping(0),
			slider_value: 0.5,
			button_clicks: 0,
			checkbox_value: false,
			string_value: String::from("Foobar! I am some text. Hee hee ho ho\na newline? 👀\n\nOh My 🦐\nاَلْعَرَبِيَّةُ"),
		}
	}

	pub fn build(&mut self, ui: &ui::Ui<'_>) {
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

		self.frame_counter += 1;
	}

	fn draw_menu_bar(&mut self, ui: &ui::Ui<'_>) {
		ui.with_horizontal_frame(|| {
			ui.button("File").with_constraints(|c| {c.margin.set_horizontal(2.0); c.padding.set_vertical(4.0); });
			ui.button("Foo Bar").with_constraints(|c| {c.margin.set_horizontal(2.0); c.padding.set_vertical(4.0); });
			ui.button("Weh").with_constraints(|c| {c.margin.set_horizontal(2.0); c.padding.set_vertical(4.0); });

			ui.spring(ui::Axis::Horizontal);

			let close_button = ui.button("x")
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
			ui.text(format!("Frame #{}", self.frame_counter))
				.with_constraints(|c| c.margin.set(8.0));

			ui.spring(ui::Axis::Horizontal);

			ui.button("Resize")
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
			c.content_alignment.set(ui::Align::Middle);
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

		let button = ui.button("");

		if button.is_clicked() {
			self.button_clicks += 1;
		}

		ui.with_parent(button, || {
			ui.text("I'm a button");

			ui.text(self.button_clicks.to_string())
				.with_style(|s| s.set_fill(ui::WidgetColorRole::Primary))
				.with_constraints(|c| {
					c.margin.set(4.0);
					c.padding.set(2.0);
				});
		});

		ui.with_horizontal_layout(|| {
			ui.text("Slider");

			ui.slider(&mut self.slider_value);

			ui.text(format!("{:.2}", self.slider_value))
				.with_style(|s| s.set_fill(ui::WidgetColorRole::PrimaryContainer))
				.with_constraints(|c| {
					c.margin.set(4.0);
					c.padding.set(4.0);
				});
		})
		.with_constraints(|c| {
			c.horizontal_size_policy.set(ui::SizingBehaviour::CAN_GROW);
			c.content_alignment.set(ui::Align::Middle);
		});

		ui.with_horizontal_layout(|| {
			ui.text("Checkbox");
			ui.checkbox(&mut self.checkbox_value);
			ui.checkbox(&mut self.checkbox_value);
			ui.checkbox(&mut self.checkbox_value);
			ui.checkbox(&mut self.checkbox_value);
		})
		.with_constraints(|c| {
			c.content_alignment.set(ui::Align::Middle);
		});

		ui.with_horizontal_layout(|| {
			ui.text("Switch");
			ui.toggle(&mut self.checkbox_value);
			ui.toggle(&mut self.checkbox_value);
			ui.toggle(&mut self.checkbox_value);
			ui.toggle(&mut self.checkbox_value);
		})
		.with_constraints(|c| {
			c.content_alignment.set(ui::Align::Middle);
		});

		ui.with_horizontal_layout(|| {
			ui.text("Text Edit");
			ui.text_edit(&mut self.string_value);
		})
		.with_constraints(|c| {
			c.content_alignment.set(ui::Align::Middle);
		});
	}
}



