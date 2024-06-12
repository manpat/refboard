use crate::ui::*;


#[derive(Debug)]
pub struct Button {}

impl Widget for Button {
	fn configure(&self, ctx: ConfigureContext<'_>) {
		ctx.constraints.horizontal_size_policy.set_default(SizingBehaviour::FIXED);
		ctx.constraints.vertical_size_policy.set_default(SizingBehaviour::FIXED);

		ctx.constraints.content_alignment.set_default(ui::Align::Middle);

		// TODO(pat.m): derive from style
		ctx.constraints.padding.set_default(8.0);
		ctx.constraints.margin.set_default(4.0);

		if ctx.style.fill.is_none() && ctx.style.outline.is_none() {
			ctx.style.set_fill(WidgetColorRole::SecondaryContainer);
		}

		// TODO(pat.m): set_default for input behaviour
		*ctx.input |= ui::InputBehaviour::OPAQUE;
	}

	fn draw(&self, ctx: DrawContext<'_>) {
		let base_color = ctx.style.text_color(ctx.app_style);
		
		let is_hovered = ctx.input.hovered_widget == Some(ctx.widget_id);

		// Paint a state layer to convey widget state
		if is_hovered {
			// TODO(pat.m): should be hot_widget/active_widget?
			let is_down = ctx.input.is_mouse_down(ui::MouseButton::Left);

			// TODO(pat.m): this calculation could be made elsewhere
			let fill_color = match is_down {
				true => base_color.with_alpha(0.32),
				false => base_color.with_alpha(0.16),
			};

			let rounding = ctx.style.rounding(ctx.app_style);

			ctx.painter.set_color(fill_color);
			ctx.painter.rounded_rect(ctx.layout.box_bounds, rounding);
		}
	}
}

impl Ui<'_> {
	pub fn button(&self, text: impl Into<String>) -> WidgetRef<'_, Button> {
		let widget = self.add_widget(Button{});

		let text = text.into();
		if !text.is_empty() {
			self.with_parent(&widget, || { self.text(text); });
		}

		widget
	}
}