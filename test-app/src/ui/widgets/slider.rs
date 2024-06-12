use crate::ui::*;

#[derive(Debug)]
pub struct Slider { pub value: f32, }

#[derive(Default, Debug)]
pub struct SliderState {
	drag_state: Option<(Vec2, f32)>,
	handle_travel_length: f32,
}

impl Widget for Slider {
	fn lifecycle(&mut self, ctx: LifecycleContext<'_>) {
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

	fn configure(&self, ctx: ConfigureContext<'_>) {
		ctx.constraints.min_width.set_default(20.0);
		ctx.constraints.preferred_height.set_default(20.0);

		ctx.constraints.padding.set_horizontal(6.0);
		ctx.constraints.padding.set_vertical(3.0);

		ctx.constraints.horizontal_size_policy.set_default(SizingBehaviour::CAN_GROW);
		ctx.constraints.vertical_size_policy.set_default(SizingBehaviour::FIXED);
	}

	fn draw(&self, ctx: DrawContext<'_>) {
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

		let primary_color = ctx.app_style.resolve_color_role(WidgetColorRole::Primary);
		let primary_container_color = ctx.app_style.resolve_color_role(WidgetColorRole::SecondaryContainer);

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

impl StatefulWidget for Slider {
	type State = SliderState;
}




impl Ui<'_> {
	pub fn slider(&self, value: &mut f32) -> WidgetRef<'_, Slider> {
		let widget = self.add_widget(Slider{ value: *value });
		*value = widget.value();
		widget
	}
}

impl WidgetRef<'_, Slider> {
	pub fn value(&self) -> f32 {
		self.widget().value
	}
}