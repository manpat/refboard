use crate::ui::*;
use cosmic_text::Edit;

// TODO(pat.m): more dynamic
const HACK_FONT_SIZE: f32 = 14.0;
const HACK_LINE_HEIGHT: f32 = HACK_FONT_SIZE; // 24.0;

// https://m3.material.io/components/text-fields/specs


#[derive(Debug)]
pub struct TextEdit {
	pub text: String,
}

#[derive(Debug)]
pub struct TextEditWidgetState {
	editor: cosmic_text::Editor<'static>,
}

impl Widget for TextEdit {
	fn lifecycle(&mut self, ctx: LifecycleContext<'_>) {
		if ctx.event == WidgetLifecycleEvent::Destroyed {
			return
		}

		let state = self.get_state_or_else(ctx.state, || TextEditWidgetState::new(ctx.text_atlas));
		state.update(ctx.text_atlas, &self.text);
	}

	fn configure(&self, ctx: ConfigureContext<'_>) {
		let state = self.get_state(ctx.state);

		ctx.constraints.margin.set_default(4.0);
		ctx.constraints.padding.set_default(8.0);

		if !ctx.constraints.min_width.is_set() || !ctx.constraints.min_height.is_set() {
			let Vec2{x: width, y: height} = state.measure(ctx.text_atlas);

			let horizontal_padding = ctx.constraints.padding.horizontal_sum();
			let vertical_padding = ctx.constraints.padding.vertical_sum();

			ctx.constraints.min_width.set_default(width + horizontal_padding);
			ctx.constraints.min_height.set_default(height + vertical_padding);
		}

		if ctx.style.fill.is_none() && ctx.style.outline.is_none() {
			ctx.style.set_fill(WidgetColorRole::SurfaceContainerHighest);
		}

		ctx.constraints.horizontal_size_policy.set_default(SizingBehaviour::FIXED);
		ctx.constraints.vertical_size_policy.set_default(SizingBehaviour::FIXED);
	}

	fn draw(&self, ctx: DrawContext<'_>) {
		let state = self.get_state(ctx.state);

		// TODO(pat.m): do layout twice - once with unbounded/max size to find the desired size for constraining
		// then again with calculated size for rendering.
		// Both of these can be cached.
		let size = ctx.layout.content_bounds.size();
		let start_pos = ctx.layout.content_bounds.min;
		state.editor.with_buffer_mut(|buffer| {
			buffer.set_size(&mut ctx.text_atlas.font_system, size.x, size.y);
		});

		// TODO(pat.m): yuck
		let text_color = match ctx.style.fill {
			Some(_) => ctx.style.text_color(ctx.app_style),
			None => ctx.app_style.resolve_color_role(ctx.text_color),
		};

		ctx.painter.set_color(text_color);
		state.draw(ctx.painter, ctx.text_atlas, start_pos);
	}
}



impl TextEditWidgetState {
	fn new(atlas: &mut TextAtlas) -> Self {
		// TODO(pat.m): derive from style
		let metrics = cosmic_text::Metrics::new(HACK_FONT_SIZE, HACK_LINE_HEIGHT);

		let font_system = &mut atlas.font_system;
		let mut buffer = cosmic_text::Buffer::new(font_system, metrics);
		buffer.set_size(font_system, 1000.0, 1000.0);
		buffer.set_wrap(font_system, cosmic_text::Wrap::None);

		let mut editor = cosmic_text::Editor::new(buffer);
		editor.shape_as_needed(font_system, true);

		TextEditWidgetState {editor}
	}

	fn update(&mut self, atlas: &mut TextAtlas, text: &str) {
		// TODO(pat.m): updating metrics from style
		// if ctx.event == WidgetLifecycleEvent::Updated {
		// 	buffer.set_metrics(metrics);
		// }

		let attrs = cosmic_text::Attrs::new();

		self.editor.with_buffer_mut(|buffer| {
			buffer.set_text(&mut atlas.font_system, text, attrs, cosmic_text::Shaping::Advanced);
		});
	}

	fn measure(&mut self, atlas: &mut TextAtlas) -> Vec2 {
		self.editor.with_buffer_mut(|buffer| {
			let buffer = buffer.borrow_with(&mut atlas.font_system);

			// TODO(pat.m): can probably cache this stuff while content/available size doesn't change
			let width = buffer.layout_runs()
				.map(|run| run.line_w)
				.max_by(|a, b| a.total_cmp(&b))
				.unwrap_or(0.0);

			let height = buffer.lines.len() as f32 * buffer.metrics().line_height;

			Vec2::new(width, height)
		})
	}

	fn draw(&mut self, painter: &mut Painter, atlas: &mut ui::TextAtlas, start_pos: impl Into<Vec2>) {
		use unicode_segmentation::UnicodeSegmentation;
		
		let start_pos = start_pos.into();

		self.editor.with_buffer(|buffer| {

			// stolen from cosmic_text
			// TODO(pat.m): make work
			// let line_height = buffer.metrics().line_height;
			// for run in buffer.layout_runs() {
			// 	let line_i = run.line_i;
			// 	let line_y = run.line_y;
			// 	let line_top = run.line_top;

			// 	if let Some((start, end)) = self.selection_bounds() {
			// 		if line_i >= start.line && line_i <= end.line {
			// 			let mut range_opt = None;
			// 			for glyph in run.glyphs.iter() {
			// 				// Guess x offset based on characters
			// 				let cluster = &run.text[glyph.start..glyph.end];
			// 				let total = cluster.grapheme_indices(true).count();
			// 				let mut c_x = glyph.x;
			// 				let c_w = glyph.w / total as f32;
			// 				for (i, c) in cluster.grapheme_indices(true) {
			// 					let c_start = glyph.start + i;
			// 					let c_end = glyph.start + i + c.len();
			// 					if (start.line != line_i || c_end > start.index)
			// 						&& (end.line != line_i || c_start < end.index)
			// 					{
			// 						range_opt = match range_opt.take() {
			// 							Some((min, max)) => Some((
			// 								cmp::min(min, c_x as i32),
			// 								cmp::max(max, (c_x + c_w) as i32),
			// 							)),
			// 							None => Some((c_x as i32, (c_x + c_w) as i32)),
			// 						};
			// 					} else if let Some((min, max)) = range_opt.take() {
			// 						f(
			// 							min,
			// 							line_top as i32,
			// 							cmp::max(0, max - min) as u32,
			// 							line_height as u32,
			// 							selection_color,
			// 						);
			// 					}
			// 					c_x += c_w;
			// 				}
			// 			}

			// 			if run.glyphs.is_empty() && end.line > line_i {
			// 				// Highlight all of internal empty lines
			// 				range_opt = Some((0, buffer.size().0 as i32));
			// 			}

			// 			if let Some((mut min, mut max)) = range_opt.take() {
			// 				if end.line > line_i {
			// 					// Draw to end of line
			// 					if run.rtl {
			// 						min = 0;
			// 					} else {
			// 						max = buffer.size().0 as i32;
			// 					}
			// 				}
			// 				f(
			// 					min,
			// 					line_top as i32,
			// 					cmp::max(0, max - min) as u32,
			// 					line_height as u32,
			// 					selection_color,
			// 				);
			// 			}
			// 		}
			// 	}
			// }

			painter.draw_text_buffer(buffer, atlas, start_pos);
		});

	}
}

impl StatefulWidget for TextEdit {
	type State = TextEditWidgetState;
}




impl Ui<'_> {
	pub fn text_edit(&self, s: &mut String) -> WidgetRef<'_, TextEdit> {
		self.add_widget(TextEdit {
			text: s.clone(),
		})
	}
}