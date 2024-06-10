use crate::prelude::*;
use std::num::Wrapping;

pub struct View {
	// item view
	// gizmo overlay
	// menus

	pub wants_quit: bool,
	pub frame_counter: Wrapping<u16>,

	pub slider_value: f32,
	pub button_clicks: u32,
}

impl View {
	pub fn new() -> View {
		View {
			wants_quit: false,
			frame_counter: Wrapping(0),
			slider_value: 0.5,
			button_clicks: 0,
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
			let slider_widget = ui.add_widget(Slider{ value: self.slider_value });
			self.slider_value = slider_widget.widget().value;

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
	}
}




#[derive(Debug)]
struct Slider { value: f32, }

#[derive(Default, Debug)]
struct SliderState {
	drag_state: Option<(Vec2, f32)>,
	handle_travel_length: f32,
}

impl ui::Widget for Slider {
	fn lifecycle(&mut self, ctx: ui::LifecycleContext<'_>) {
		let state = self.get_state_or_default(ctx.state);

		let is_active = ctx.input.active_widget == Some(ctx.widget_id);

		if is_active {
			if let Some((start_pos, start_value)) = state.drag_state {
				let delta = ctx.input.cursor_pos_view.unwrap_or(start_pos) - start_pos;

				self.value = (start_value + delta.x / state.handle_travel_length).clamp(0.0, 1.0);

			} else {
				state.drag_state = Some((ctx.input.cursor_pos_view.unwrap(), self.value));
			}
		} else {
			state.drag_state = None;
		}
	}

	fn configure(&self, ctx: ui::ConfigureContext<'_>) {
		ctx.constraints.min_width.set_default(20.0);
		ctx.constraints.preferred_height.set_default(20.0);

		ctx.constraints.padding.set_horizontal(6.0);
		ctx.constraints.padding.set_vertical(3.0);

		ctx.constraints.horizontal_size_policy.set_default(ui::SizingBehaviour::CAN_GROW);
		ctx.constraints.vertical_size_policy.set_default(ui::SizingBehaviour::FIXED);
	}

	fn draw(&self, ctx: ui::DrawContext<'_>) {
		let state = self.get_state(ctx.state);

		let bounds = ctx.layout.content_bounds;
		let half_height = bounds.height() / 2.0;

		let handle_stroke_width = 5.0;
		let track_width = 12.0;
		let dot_radius = track_width / 4.0;

		let handle_travel_start = bounds.min.x + handle_stroke_width;
		let handle_travel_length = bounds.width() - handle_stroke_width * 2.0;

		state.handle_travel_length = handle_travel_length;

		let handle_pos_x = handle_travel_start + handle_travel_length * self.value;

		let left_center = bounds.min + Vec2::from_y(half_height);
		let right_center = bounds.max - Vec2::from_y(half_height);
		let handle_center = Vec2::new(handle_pos_x, left_center.y);

		let left_stop = Vec2::new(handle_travel_start, left_center.y);
		let right_stop = Vec2::new(handle_travel_start + handle_travel_length, left_center.y);

		let primary_color = ctx.app_style.resolve_color_role(ui::WidgetColorRole::Primary);
		let primary_container_color = ctx.app_style.resolve_color_role(ui::WidgetColorRole::SecondaryContainer);

		let active_track_color = primary_color;
		let mut inactive_track_color = primary_container_color;

		// Apply state layer
		let is_hovered = ctx.input.hovered_widget == Some(ctx.widget_id);
		let is_active = ctx.input.active_widget == Some(ctx.widget_id);

		if is_active {
			inactive_track_color = 0.16f32.lerp(inactive_track_color, primary_color);
		} else if is_hovered {
			inactive_track_color = 0.08f32.lerp(inactive_track_color, primary_color);
		}

		let handle_color = active_track_color;

		ctx.painter.set_line_width(track_width);
		ctx.painter.stroke_options.start_cap = lyon::tessellation::LineCap::Round;
		ctx.painter.stroke_options.end_cap = lyon::tessellation::LineCap::Butt;

		// Active Track
		ctx.painter.set_color(active_track_color);
		ctx.painter.line(left_center, handle_center - Vec2::from_x(handle_stroke_width));

		// Inactive Track
		ctx.painter.set_color(inactive_track_color);
		ctx.painter.line(right_center, handle_center + Vec2::from_x(handle_stroke_width));

		// Active Track Stop
		ctx.painter.set_color(inactive_track_color);
		ctx.painter.circle(left_stop, dot_radius);

		// Inactive Track Stop
		ctx.painter.set_color(active_track_color);
		ctx.painter.circle(right_stop, dot_radius);

		// Handle
		ctx.painter.set_line_width(handle_stroke_width);
		ctx.painter.stroke_options.start_cap = lyon::tessellation::LineCap::Round;
		ctx.painter.stroke_options.end_cap = lyon::tessellation::LineCap::Round;

		let cross_size = Vec2::from_y(half_height);

		ctx.painter.set_color(handle_color);
		ctx.painter.line(handle_center - cross_size, handle_center + cross_size);
	}
}

impl ui::StatefulWidget for Slider {
	type State = SliderState;
}