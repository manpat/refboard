use crate::ui::*;
use cosmic_text::{Edit, /*Cursor*/};

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
	origin: Vec2,
	active: bool,
}

impl Widget for TextEdit {
	fn lifecycle(&mut self, mut ctx: LifecycleContext<'_>) {
		if ctx.event == WidgetLifecycleEvent::Destroyed {
			return
		}

		let state = self.get_state_or_else(ctx.state, || TextEditWidgetState::new(ctx.text_atlas));

		state.update(ctx.text_atlas, &self.text);
		if self.handle_input(&mut ctx) {
			let state = self.get_state(ctx.state);

			self.text.clear();

			state.editor.with_buffer(|buffer| {
				for line in buffer.lines.iter() {
					self.text.push_str(line.text());
					self.text.push('\n');
				}

				// Pop the final newline if there is one
				self.text.pop();
			})
		}
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
		state.draw(ctx.painter, ctx.text_atlas, start_pos, ctx.app_style);
	}
}


impl TextEdit {
	fn handle_input(&mut self, ctx: &mut LifecycleContext<'_>) -> bool {
		use cosmic_text::Action;

		let state = self.get_state(ctx.state);

		let is_hovered = ctx.input.hovered_widget == Some(ctx.widget_id);
		let is_focussed = ctx.input.focus_widget == Some(ctx.widget_id);
		let is_active = ctx.input.active_widget == Some(ctx.widget_id);

		state.active = is_focussed || is_active;

		if !state.active {
			return false
		}

		let Some(mouse_pos) = ctx.input.cursor_pos else { return false };
		let relative_mouse = mouse_pos - state.origin;

		if is_hovered {
			if ctx.input.was_mouse_pressed(ui::MouseButton::Left) {
				state.editor.action(&mut ctx.text_atlas.font_system, Action::Click {
					x: relative_mouse.x as i32,
					y: relative_mouse.y as i32,
				});
			}

			if let Some(_) = ctx.input.mouse_drag_delta(ui::MouseButton::Left) {
				state.editor.action(&mut ctx.text_atlas.font_system, Action::Drag {
					x: relative_mouse.x as i32,
					y: relative_mouse.y as i32,
				});
			}
		}

		for event in ctx.input.keyboard_input.iter() {
			match event {
				ui::KeyboardEvent::Character(ch) => {
					state.editor.action(&mut ctx.text_atlas.font_system, Action::Insert(*ch));
				}

				_ => {}
			}
		}

		state.editor.shape_as_needed(&mut ctx.text_atlas.font_system, true);

		true
	}
}


impl TextEditWidgetState {
	fn new(atlas: &mut TextAtlas) -> Self {
		// TODO(pat.m): derive from style
		let metrics = cosmic_text::Metrics::new(HACK_FONT_SIZE, HACK_LINE_HEIGHT);

		let font_system = &mut atlas.font_system;
		let mut buffer = cosmic_text::Buffer::new(font_system, metrics);
		buffer.set_size(font_system, f32::INFINITY, 1000.0);
		buffer.set_wrap(font_system, cosmic_text::Wrap::None);

		let editor = cosmic_text::Editor::new(buffer);

		TextEditWidgetState {
			editor,
			origin: Vec2::zero(),
			active: false,
		}
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

	fn draw(&mut self, painter: &mut Painter, atlas: &mut ui::TextAtlas, start_pos: impl Into<Vec2>, app_style: &AppStyle) {
		let start_pos = start_pos.into();
		let text_color = painter.color;

		self.editor.with_buffer(|buffer| {
			// Draw selection highlight and cursor
			if self.active {
				for run in buffer.layout_runs() {
					self.draw_selection_for_run(painter, &run, start_pos, app_style);
					self.draw_cursor_for_run(painter, &run, start_pos);
				}
			}

			// TODO(pat.m): make selection bounds effect text colour
			painter.set_color(text_color);
			painter.draw_text_buffer(buffer, atlas, start_pos);
		});

		self.origin = start_pos;
	}

	fn draw_selection_for_run(&self, painter: &mut Painter, run: &cosmic_text::LayoutRun<'_>, start_pos: Vec2, app_style: &AppStyle) {
		let Some((cursor_start, cursor_end)) = self.editor.selection_bounds() else {
			return
		};

		let line_index = run.line_i;
		if line_index < cursor_start.line || line_index > cursor_end.line {
			return
		}

		let line_top = run.line_top;
		let line_height = self.editor.with_buffer(|buffer| buffer.metrics().line_height);

		let empty_indicator_width = 5.0;

		let line_top_left = start_pos + Vec2::from_y(line_top);
		let line_bottom_left = line_top_left + Vec2::from_y(line_height);

		let selection_extends_to_next_line = line_index < cursor_end.line;

		let (start_x, mut width) = run.highlight(cursor_start, cursor_end).unwrap_or((0.0, 0.0));
		if selection_extends_to_next_line {
			width += empty_indicator_width;
		}

		let min = line_top_left + Vec2::from_x(start_x);
		let max = line_bottom_left + Vec2::from_x(start_x + width);

		let selection_color = app_style.resolve_color_role(WidgetColorRole::OnSurface)
			.with_alpha(0.1);

		painter.set_color(selection_color);
		painter.rect(Aabb2{min, max});
	}

	// Heavily adapted from cosmic_text::Editor::draw
	fn draw_cursor_for_run(&self, painter: &mut Painter, run: &cosmic_text::LayoutRun<'_>, start_pos: Vec2) {
		use unicode_segmentation::UnicodeSegmentation;

		let cursor = self.editor.cursor();

		// Cursor isn't on this line
		if cursor.line != run.line_i {
			return;
		}

		// TODO(pat.m): next release of cosmic_text should have an Editor::cursor_position which will mean we can
		// delete most of this.

		// Search for the index of the glyph after the cursor, or if the cursor is within a cluster
		// the index of the containing glyph and an approximate offset indicating which grapheme within the cluster.
		let (glyph_index, glyph_offset) = 'search: {
			for (glyph_i, glyph) in run.glyphs.iter().enumerate() {
				// Exact match, nice and easy
				if cursor.index == glyph.start {
					break 'search (glyph_i, 0.0);

				// Cursor is within a cluster, estimate offset.
				} else if cursor.index > glyph.start && cursor.index < glyph.end {
					let mut before = 0;
					let mut total = 0;

					let cluster = &run.text[glyph.start..glyph.end];
					for (i, _) in cluster.grapheme_indices(true) {
						if glyph.start + i < cursor.index {
							before += 1;
						}
						total += 1;
					}

					let offset = glyph.w * (before as f32) / (total as f32);
					break 'search (glyph_i, offset);
				}
			}

			// cursor.index > num glyphs, so snap it to the end of the last glyph if there is one,
			// otherwise the line is empty so the start of the line is the only reasonable place.
			match run.glyphs.last() {
				Some(glyph) => {
					if cursor.index == glyph.end {
						break 'search (run.glyphs.len(), 0.0);
					}
				}
				None => {
					break 'search (0, 0.0);
				}
			}

			// If we can't find what we're looking for, then bail entirely
			return;
		};


		// Get the glyph found above and calculate an appropriate position for the cursor.
		let cursor_x = match run.glyphs.get(glyph_index) {
			Some(glyph) => {
				// Start of detected glyph
				if glyph.level.is_rtl() {
					glyph.x + glyph.w - glyph_offset
				} else {
					glyph.x + glyph_offset
				}
			}

			None => match run.glyphs.last() {
				Some(glyph) => {
					// End of last glyph
					if glyph.level.is_rtl() {
						glyph.x
					} else {
						glyph.x + glyph.w
					}
				}

				None => {
					// Start of empty line
					0.0
				}
			}
		};

		let line_height = self.editor.with_buffer(|buffer| buffer.metrics().line_height);

		let line_top = run.line_top;
		let start = start_pos + Vec2::new(cursor_x, line_top);
		let end = start_pos + Vec2::new(cursor_x, line_top + line_height);

		// TODO(pat.m): cursor color
		painter.set_color([1.0; 4]);
		painter.set_line_width(1.0);
		painter.line(start, end);
	}
}

impl StatefulWidget for TextEdit {
	type State = TextEditWidgetState;
}




impl Ui<'_> {
	pub fn text_edit(&self, s: &mut String) -> WidgetRef<'_, TextEdit> {
		let widget = self.add_widget(TextEdit {
			text: std::mem::take(s),
		});

		*s = std::mem::take(&mut widget.widget().text);

		widget
	}
}