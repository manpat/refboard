use crate::ui::*;


// TODO(pat.m): more dynamic
const HACK_FONT_SIZE: f32 = 14.0;
const HACK_LINE_HEIGHT: f32 = HACK_FONT_SIZE; // 24.0;



#[derive(Debug)]
pub struct Text {
	pub text: String,
}

#[derive(Debug)]
pub struct TextWidgetState {
	buffer: cosmic_text::Buffer,
}

impl Widget for Text {
	fn lifecycle(&mut self, ctx: LifecycleContext<'_>) {
		if ctx.event == WidgetLifecycleEvent::Destroyed {
			return
		}

		let state = self.get_state_or_else(ctx.state, || TextWidgetState::new(ctx.text_atlas));
		state.update(ctx.text_atlas, &self.text);
	}

	fn configure(&self, ctx: ConfigureContext<'_>) {
		let state = self.get_state(ctx.state);

		if !ctx.constraints.min_width.is_set() || !ctx.constraints.min_height.is_set() {
			let Vec2{x: width, y: height} = state.measure(ctx.text_atlas);

			let horizontal_padding = ctx.constraints.padding.horizontal_sum();
			let vertical_padding = ctx.constraints.padding.vertical_sum();

			ctx.constraints.min_width.set_default(width + horizontal_padding);
			ctx.constraints.min_height.set_default(height + vertical_padding);
		}

		ctx.constraints.horizontal_size_policy.set_default(SizingBehaviour::FIXED);
		ctx.constraints.vertical_size_policy.set_default(SizingBehaviour::FIXED);

		// TODO(pat.m): set_default for input behaviour
		*ctx.input |= ui::InputBehaviour::TRANSPARENT;
	}

	fn draw(&self, ctx: DrawContext<'_>) {
		let state = self.get_state(ctx.state);

		// TODO(pat.m): do layout twice - once with unbounded/max size to find the desired size for constraining
		// then again with calculated size for rendering.
		// Both of these can be cached.
		let size = ctx.layout.content_bounds.size();
		let start_pos = ctx.layout.content_bounds.min;
		state.buffer.set_size(&mut ctx.text_atlas.font_system, size.x, size.y);

		// TODO(pat.m): yuck
		let text_color = match ctx.style.fill {
			Some(_) => ctx.style.text_color(ctx.app_style),
			None => ctx.app_style.resolve_color_role(ctx.text_color),
		};

		ctx.painter.set_color(text_color);
		ctx.painter.draw_text_buffer(&state.buffer, ctx.text_atlas, start_pos);
	}
}



impl TextWidgetState {
	fn new(atlas: &mut TextAtlas) -> Self {
		// TODO(pat.m): derive from style
		let metrics = cosmic_text::Metrics::new(HACK_FONT_SIZE, HACK_LINE_HEIGHT);

		let font_system = &mut atlas.font_system;
		let mut buffer = cosmic_text::Buffer::new(font_system, metrics);
		buffer.set_size(font_system, 1000.0, 1000.0);
		buffer.set_wrap(font_system, cosmic_text::Wrap::None);
		TextWidgetState {buffer}
	}

	fn update(&mut self, atlas: &mut TextAtlas, text: &str) {
		// TODO(pat.m): updating metrics from style
		// if ctx.event == WidgetLifecycleEvent::Updated {
		// 	buffer.set_metrics(metrics);
		// }

		let attrs = cosmic_text::Attrs::new();

		let mut buffer = self.buffer.borrow_with(&mut atlas.font_system);
		buffer.set_text(text, attrs, cosmic_text::Shaping::Advanced);
	}

	fn measure(&mut self, atlas: &mut TextAtlas) -> Vec2 {
		let buffer = self.buffer.borrow_with(&mut atlas.font_system);

		// TODO(pat.m): can probably cache this stuff while content/available size doesn't change
		let width = buffer.layout_runs()
			.map(|run| run.line_w)
			.max_by(|a, b| a.total_cmp(&b))
			.unwrap_or(0.0);

		let height = buffer.lines.len() as f32 * buffer.metrics().line_height;

		Vec2::new(width, height)
	}
}

impl StatefulWidget for Text {
	type State = TextWidgetState;
}




impl Ui<'_> {
	pub fn text(&self, s: impl Into<String>) -> WidgetRef<'_, Text> {
		self.add_widget(Text {
			text: s.into(),
		})
	}
}