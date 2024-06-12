use crate::ui::*;


#[derive(Debug)]
pub struct Toggle { pub value: bool }

impl Widget for Toggle {
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

		ctx.constraints.preferred_width.set_default(40.0);
		ctx.constraints.preferred_height.set_default(24.0);

		// TODO(pat.m): derive from style
		ctx.constraints.padding.set_default(2.0);
		ctx.constraints.margin.set_default(4.0);

		if ctx.style.fill.is_none() && ctx.style.outline.is_none() {
			ctx.style.set_rounding(ctx.constraints.preferred_height.get() / 2.0);

			if self.value {
				ctx.style.set_fill(WidgetColorRole::Primary);
			} else {
				ctx.style.set_fill(WidgetColorRole::SurfaceContainerHighest);
				ctx.style.set_outline(WidgetColorRole::Outline);
			}
		}

		// TODO(pat.m): set_default for input behaviour
		*ctx.input |= ui::InputBehaviour::OPAQUE;
	}

	fn draw(&self, ctx: DrawContext<'_>) {
		let rounding = ctx.style.rounding(ctx.app_style);
		
		let is_hovered = ctx.input.hovered_widget == Some(ctx.widget_id);
		let is_active = ctx.input.active_widget == Some(ctx.widget_id);

		let bounds = ctx.layout.content_bounds;
		let base_radius = bounds.height() / 2.0;

		// TODO(pat.m): animate :)
		let position = match self.value {
			false => bounds.min + Vec2::splat(base_radius),
			true => bounds.max - Vec2::splat(base_radius),
		};

		let handle_color = match self.value {
			false => ctx.app_style.resolve_color_role(WidgetColorRole::Outline),
			true => ctx.app_style.resolve_color_role(WidgetColorRole::OnPrimary),
		};

		let handle_radius = match (self.value, is_active) {
			(_, true) => base_radius * 8.0 / 7.0,
			(true, _) => base_radius,
			(false, _) => base_radius * 7.0 / 8.0,
		};

		ctx.painter.set_color(handle_color);
		ctx.painter.circle(position, handle_radius);

		// Paint a state layer to convey widget state
		if is_hovered || is_active {
			// TODO(pat.m): this calculation could be made elsewhere
			let base_color = ctx.style.text_color(ctx.app_style);
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
	pub fn toggle(&self, value: &mut bool) -> WidgetRef<'_, Toggle> {
		let widget = self.add_widget(Toggle{ value: *value });
		*value = widget.is_checked();
		widget
	}
}

impl WidgetRef<'_, Toggle> {
	pub fn is_checked(&self) -> bool {
		self.widget().value
	}
}