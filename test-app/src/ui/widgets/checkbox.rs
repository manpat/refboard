use crate::ui::*;


#[derive(Debug)]
pub struct Checkbox { pub value: bool }

impl Widget for Checkbox {
	fn lifecycle(&mut self, ctx: LifecycleContext<'_>) {
		let is_hovered = ctx.input.hovered_widget == Some(ctx.widget_id);
		if is_hovered && ctx.input.was_mouse_released(MouseButton::Left) {
			self.value = !self.value;
			ctx.trigger_redraw();
		}
	}

	fn configure(&self, ctx: ConfigureContext<'_>) {
		ctx.constraints.horizontal_size_policy.set_default(SizingBehaviour::FIXED);
		ctx.constraints.vertical_size_policy.set_default(SizingBehaviour::FIXED);

		ctx.constraints.preferred_width.set_default(20.0);
		ctx.constraints.preferred_height.set_default(20.0);

		// TODO(pat.m): derive from style
		ctx.constraints.padding.set_default(3.0);
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

		// Paint a state layer to convey widget state
		if is_hovered {
			// TODO(pat.m): should be hot_widget/active_widget?
			let is_down = ctx.input.is_mouse_down(ui::MouseButton::Left);

			// TODO(pat.m): this calculation could be made elsewhere
			let fill_color = match is_down {
				false => base_color.with_alpha(0.16),
				true => base_color.with_alpha(0.32),
			};

			ctx.painter.set_color(fill_color);
			ctx.painter.rounded_rect(ctx.layout.box_bounds, rounding);
		}

		// Checked state
		if self.value {
			ctx.painter.set_color(base_color);
			ctx.painter.rounded_rect(ctx.layout.content_bounds, 2.0);
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