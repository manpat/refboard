use crate::ui::*;


#[derive(Debug)]
pub struct Checkbox { pub value: bool }

impl Widget for Checkbox {
	fn lifecycle(&mut self, ctx: LifecycleContext<'_>) {
		let is_hovered = ctx.input.hovered_widget == Some(ctx.widget_id);
		// TODO(pat.m): should use active_widget, not hovered widget
		if is_hovered && ctx.input.was_mouse_released(MouseButton::Left) {
			self.value = !self.value;
			ctx.trigger_redraw();
		}
	}

	fn configure(&self, ctx: ConfigureContext<'_>) {
		ctx.constraints.horizontal_size_policy.set_default(SizingBehaviour::FIXED);
		ctx.constraints.vertical_size_policy.set_default(SizingBehaviour::FIXED);

		ctx.constraints.preferred_width.set_default(18.0);
		ctx.constraints.preferred_height.set_default(18.0);

		// TODO(pat.m): derive from style
		ctx.constraints.padding.set_default(4.0);
		ctx.constraints.margin.set_default(4.0);

		if ctx.style.fill.is_none() && ctx.style.outline.is_none() {
			if self.value {
				ctx.style.set_fill(WidgetColorRole::Primary);
			} else {
				ctx.style.set_fill(WidgetColorRole::SecondaryContainer);
			}
		}

		// TODO(pat.m): set_default for input behaviour
		*ctx.input |= ui::InputBehaviour::OPAQUE;
	}

	fn draw(&self, ctx: DrawContext<'_>) {
		let base_color = ctx.style.text_color(ctx.app_style);
		let rounding = ctx.style.rounding(ctx.app_style);
		
		let is_hovered = ctx.input.hovered_widget == Some(ctx.widget_id);
		let is_active = ctx.input.active_widget == Some(ctx.widget_id);

		// Checked state
		if self.value {
			let bounds = ctx.layout.content_bounds;

			ctx.painter.set_color(base_color);
			ctx.painter.set_line_width(3.0);

			ctx.painter.stroke_options.start_cap = lyon::tessellation::LineCap::Butt;
			ctx.painter.stroke_options.end_cap = lyon::tessellation::LineCap::Butt;

			ctx.painter.line(bounds.min, bounds.max);
			ctx.painter.line(bounds.min_max_corner(), bounds.max_min_corner());
		}

		// Paint a state layer to convey widget state
		if is_hovered || is_active {
			// TODO(pat.m): this calculation could be made elsewhere
			let fill_color = match is_active {
				false => base_color.with_alpha(0.16),
				true => base_color.with_alpha(0.32),
			};

			ctx.painter.set_color(fill_color);
			ctx.painter.rounded_rect(ctx.layout.box_bounds, rounding);
		}
	}
}

impl Ui<'_> {
	pub fn checkbox(&self, value: &mut bool) -> WidgetRef<'_, Checkbox> {
		let widget = self.add_widget(Checkbox{ value: *value });
		*value = widget.is_checked();
		widget
	}
}

impl WidgetRef<'_, Checkbox> {
	pub fn is_checked(&self) -> bool {
		self.widget().value
	}
}