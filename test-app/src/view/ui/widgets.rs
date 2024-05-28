// use crate::prelude::*;
use crate::view::ui::*;

use std::fmt::{self, Debug};



impl Ui<'_> {
	pub fn dummy(&self) -> WidgetRef<'_, ()> {
		self.add_widget(())
	}

	pub fn spring(&self, axis: Axis) -> WidgetRef<'_, Spring> {
		// TODO(pat.m): can I derive Axis from context?
		self.add_widget(Spring(axis))
	}

	pub fn button(&self) -> WidgetRef<'_, Button> {
		self.add_widget(Button)
	}

	pub fn text(&self, s: impl Into<String>) -> WidgetRef<'_, Text> {
		self.add_widget(Text {
			text: s.into(),
			color: Color::white(),
		})
	}
}



impl Widget for () {
	fn constrain(&self, ctx: ConstraintContext<'_>) {
		ctx.constraints.margin.set_default(4.0);
		ctx.constraints.padding.set_default(8.0);

		ctx.constraints.horizontal_size_policy.set_default(SizingBehaviour::FLEXIBLE);
	}

	fn draw(&self, ctx: DrawContext<'_>) {
		ctx.painter.set_color([0.5; 3]);
		ctx.painter.rounded_rect_outline(ctx.layout.box_bounds, 8.0);
		ctx.painter.line(ctx.layout.box_bounds.min, ctx.layout.box_bounds.max);
		ctx.painter.line(ctx.layout.box_bounds.min_max_corner(), ctx.layout.box_bounds.max_min_corner());

		ctx.painter.set_color([0.5, 1.0, 0.5, 0.5]);
		ctx.painter.rounded_rect_outline(ctx.layout.content_bounds, 8.0);
	}
}




pub struct DrawFnWidget<F>(pub F)
	where F: Fn(DrawContext<'_>) + 'static;

impl<F> Widget for DrawFnWidget<F>
	where F: Fn(DrawContext<'_>) + 'static
{
	fn draw(&self, ctx: DrawContext<'_>) {
		(self.0)(ctx);
	}
}

impl<F> Debug for DrawFnWidget<F>
	where F: Fn(DrawContext<'_>) + 'static
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "DrawFnWidget()")
	}
}



#[derive(Debug)]
pub struct BoxLayout {
	pub axis: Axis,
}

impl BoxLayout {
	pub fn horizontal() -> Self {
		BoxLayout { axis: Axis::Horizontal }
	}

	pub fn vertical() -> Self {
		BoxLayout { axis: Axis::Vertical }
	}
}

impl Widget for BoxLayout {
	fn constrain(&self, ctx: ConstraintContext<'_>) {
		let ConstraintContext { constraints, children, constraint_map, .. } = ctx;

		constraints.layout_axis.set_default(self.axis);

		let main_axis = constraints.layout_axis.get();
		let cross_axis = main_axis.opposite();

		let mut total_main_min_length = 0.0f32;
		let mut total_cross_min_length = 0.0f32;

		let mut total_main_preferred_length = 0.0f32;
		let mut total_cross_preferred_length = 0.0f32;

		for &child in children {
			let child_constraints = &constraint_map[&child];

			let margin_main = child_constraints.margin.axis_sum(main_axis);
			let margin_cross = child_constraints.margin.axis_sum(cross_axis);

			let min_length = child_constraints.min_length(main_axis);
			let preferred_length = child_constraints.preferred_length(main_axis);

			total_main_min_length += min_length + margin_main;
			total_main_preferred_length += preferred_length + margin_main;

			total_cross_min_length = total_cross_min_length.max(child_constraints.min_length(cross_axis) + margin_cross);
			total_cross_preferred_length = total_cross_preferred_length.max(child_constraints.preferred_length(cross_axis) + margin_cross);
		}

		constraints.padding.set_default(8.0);

		let padding_main = constraints.padding.axis_sum(main_axis);
		let padding_cross = constraints.padding.axis_sum(cross_axis);

		total_main_min_length += padding_main;
		total_cross_min_length += padding_cross;

		total_main_preferred_length += padding_main;
		total_cross_preferred_length += padding_cross;

		constraints.min_length_mut(main_axis).set_default(total_main_min_length);
		constraints.min_length_mut(cross_axis).set_default(total_cross_min_length);

		constraints.preferred_length_mut(main_axis).set_default(total_main_preferred_length);
		constraints.preferred_length_mut(cross_axis).set_default(total_cross_preferred_length);

		constraints.size_policy_mut(main_axis).set_default(SizingBehaviour::FIXED);
		constraints.size_policy_mut(cross_axis).set_default(SizingBehaviour::FIXED);
	}
}



#[derive(Debug)]
pub struct FrameWidget<I: Widget> {
	pub inner: I,

	pub outline_color: Color,
	pub background_color: Color,
}

impl<W: Widget> FrameWidget<W> {
	pub fn new(inner: W) -> Self {
		FrameWidget {
			inner,
			outline_color: Color::grey_a(1.0, 0.04),
			background_color: Color::grey_a(1.0, 0.01),
		}
	}

	pub fn with_color(mut self, color: impl Into<Color>) -> Self {
		self.background_color = color.into();
		self
	}
}

impl FrameWidget<BoxLayout> {
	pub fn horizontal() -> Self {
		FrameWidget::new(BoxLayout::horizontal())
	}

	pub fn vertical() -> Self {
		FrameWidget::new(BoxLayout::vertical())
	}
}

impl<W> Widget for FrameWidget<W>
	where W: Widget
{
	fn constrain(&self, ctx: ConstraintContext<'_>) {
		self.inner.constrain(ctx);
	}

	fn draw(&self, ctx: DrawContext<'_>) {
		let rounding = 4.0;

		ctx.painter.set_color(self.background_color);
		ctx.painter.rounded_rect(ctx.layout.box_bounds, rounding);

		ctx.painter.set_color(self.outline_color);
		ctx.painter.rounded_rect_outline(ctx.layout.box_bounds, rounding);

		self.inner.draw(ctx);
	}
}




#[derive(Debug)]
pub struct Spring(pub Axis);

impl Widget for Spring {
	fn constrain(&self, ctx: ConstraintContext<'_>) {
		ctx.constraints.size_policy_mut(self.0).set_default(SizingBehaviour::FLEXIBLE);
		ctx.constraints.size_policy_mut(self.0.opposite()).set_default(SizingBehaviour::FIXED);
		ctx.constraints.self_alignment.set_default(Align::Middle);
	}
}



#[derive(Debug)]
pub struct Button;

impl Widget for Button {
	fn constrain(&self, ctx: ConstraintContext<'_>) {
		ctx.constraints.padding.set_default(8.0);
		ctx.constraints.margin.set_default(4.0);

		ctx.constraints.min_width.set_default(72.0);
		ctx.constraints.min_height.set_default(32.0);
	}

	fn draw(&self, ctx: DrawContext<'_>) {
		let rounding = 4.0;

		ctx.painter.set_color(Color::grey(0.3));
		ctx.painter.rounded_rect(ctx.layout.box_bounds, rounding);

		ctx.painter.set_color(Color::grey(0.5));
		ctx.painter.rounded_rect_outline(ctx.layout.box_bounds, rounding);
	}
}



// TODO(pat.m): more dynamic
const HACK_FONT_SIZE: f32 = 14.0;
const HACK_LINE_HEIGHT: f32 = HACK_FONT_SIZE; // 24.0;



#[derive(Debug)]
pub struct Text {
	pub text: String,
	pub color: Color,
}

#[derive(Debug)]
struct TextState {
	buffer: cosmic_text::Buffer,
}

impl Widget for Text {
	fn lifecycle(&mut self, ctx: LifecycleContext<'_>) {
		if ctx.event == WidgetLifecycleEvent::Destroyed {
			return
		}

		let font_system = &mut ctx.text_state.font_system;

		let metrics = cosmic_text::Metrics::new(HACK_FONT_SIZE, HACK_LINE_HEIGHT);

		if ctx.event == WidgetLifecycleEvent::Created {
			let mut buffer = cosmic_text::Buffer::new(font_system, metrics);
			buffer.set_size(font_system, 1000.0, 1000.0);
			buffer.set_wrap(font_system, cosmic_text::Wrap::None);
			ctx.state.set(TextState {buffer});
		}

		let state = ctx.state.get::<TextState>();
		let mut buffer = state.buffer.borrow_with(font_system);

		// if ctx.event == WidgetLifecycleEvent::Updated {
		// 	buffer.set_metrics(metrics);
		// }

		let attrs = cosmic_text::Attrs::new();
		buffer.set_text(&self.text, attrs, cosmic_text::Shaping::Advanced);
	}

	fn constrain(&self, ctx: ConstraintContext<'_>) {
		if !ctx.constraints.min_width.is_set() {
			let state = ctx.state.get::<TextState>();
			let buffer = state.buffer.borrow_with(&mut ctx.text_state.font_system);

			let min_width = buffer.layout_runs()
				.map(|run| run.line_w)
				.max_by(|a, b| a.total_cmp(&b))
				.unwrap_or(0.0);

			ctx.constraints.min_width.set_default(min_width);
		}

		if !ctx.constraints.min_height.is_set() {
			let state = ctx.state.get::<TextState>();
			let buffer = state.buffer.borrow_with(&mut ctx.text_state.font_system);

			let min_height = buffer.lines.len() as f32 * HACK_LINE_HEIGHT;

			ctx.constraints.min_height.set_default(min_height);
		}

		ctx.constraints.horizontal_size_policy.set_default(SizingBehaviour::FIXED);
		ctx.constraints.vertical_size_policy.set_default(SizingBehaviour::FIXED);
	}

	fn draw(&self, ctx: DrawContext<'_>) {
		let state = ctx.state.get::<TextState>();

		let font_system = &mut ctx.text_state.font_system;

		// TODO(pat.m): do layout twice - once with unbounded/max size to find the desired size for constraining
		// then again with calculated size for rendering.
		// Both of these can be cached.
		let size = ctx.layout.content_bounds.size();
		let start_pos = ctx.layout.content_bounds.min;
		state.buffer.set_size(font_system, size.x, size.y);

		// TODO(pat.m): move into painter
		for run in state.buffer.layout_runs() {
			for glyph in run.glyphs.iter() {
				let physical_glyph = glyph.physical(start_pos.to_tuple(), 1.0);

				let glyph_info = ctx.text_state.request_glyph(&physical_glyph.cache_key);
				// TODO(pat.m): pass glyph_info.subpixel_alpha down to painter

				let pos = Vec2::new(physical_glyph.x as f32 + glyph_info.offset_x, physical_glyph.y as f32 + glyph_info.offset_y + run.line_y);
				let uv_pos = Vec2::new(glyph_info.uv_x, glyph_info.uv_y);
				let size = Vec2::new(glyph_info.width, glyph_info.height);
				let bounds = Aabb2::new(pos, pos + size);

				let atlas_size = Vec2::splat(2048.0);
				let uv_bounds = Aabb2::new(uv_pos / atlas_size, (uv_pos + size) / atlas_size);

				let glyph_color = match (glyph.color_opt, glyph_info.is_color_bitmap) {
					(_, true) => Color::white(),
					(Some(ct_color), _) => Color::from(ct_color.as_rgba()).to_linear(),
					(None, _) => self.color,
				};

				ctx.painter.set_color(glyph_color);
				ctx.painter.set_uv_rect(uv_bounds);
				ctx.painter.rect(bounds);
			}
		}

		ctx.painter.set_uv_rect(None);
	}
}

