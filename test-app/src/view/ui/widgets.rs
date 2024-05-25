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
		self.add_widget(Text(s.into()))
	}
}



impl Widget for () {
	fn constrain(&self, ctx: ConstraintContext<'_>) {
		ctx.constraints.margin.set_default(4.0);
		ctx.constraints.padding.set_default(8.0);

		ctx.constraints.horizontal_size_policy.set_default(SizingBehaviour::FLEXIBLE);
	}

	fn draw(&self, ctx: DrawContext<'_>) {
		let widget_color = [0.5; 3];
		ctx.painter.rounded_rect_outline(ctx.layout.box_bounds, 8.0, widget_color);
		ctx.painter.line(ctx.layout.box_bounds.min, ctx.layout.box_bounds.max, widget_color);
		ctx.painter.line(ctx.layout.box_bounds.min_max_corner(), ctx.layout.box_bounds.max_min_corner(), widget_color);

		ctx.painter.rounded_rect_outline(ctx.layout.content_bounds, 8.0, [0.5, 1.0, 0.5, 0.5]);
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

		ctx.painter.rounded_rect(ctx.layout.box_bounds, rounding, self.background_color);
		ctx.painter.rounded_rect_outline(ctx.layout.box_bounds, rounding, self.outline_color);

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

		ctx.painter.rounded_rect(ctx.layout.box_bounds, rounding, Color::grey(0.3));
		ctx.painter.rounded_rect_outline(ctx.layout.box_bounds, rounding, Color::grey(0.5));
	}
}



// TODO(pat.m): more dynamic
const HACK_FONT_SIZE: f32 = 20.0;
const HACK_LINE_HEIGHT: f32 = 32.0;



#[derive(Debug)]
pub struct Text(pub String);

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
		buffer.set_text(&self.0, attrs, cosmic_text::Shaping::Advanced);
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

		ctx.painter.rect_outline(ctx.layout.box_bounds, Color::grey_a(0.5, 0.1));
		// ctx.painter.rect(ctx.layout.content_bounds, Color::grey(1.0));

		let font_system = &mut ctx.text_state.font_system;

		let size = ctx.layout.content_bounds.size();
		let start_pos = ctx.layout.content_bounds.min;
		state.buffer.set_size(font_system, size.x, size.y);

		for run in state.buffer.layout_runs() {
			for glyph in run.glyphs.iter() {
				let physical_glyph = glyph.physical(start_pos.to_tuple(), 1.0);

				// TODO(pat.m): check local cache first - otherwise stage returned image for text atlas upload
				// text_state.glyph_info(physical_glyph.cache_key)
				let Some(image) = ctx.text_state.swash_cache.get_image(font_system, physical_glyph.cache_key)
					else { continue };

				let placement = image.placement;

				let pos = Vec2::new(physical_glyph.x as f32 + placement.left as f32, physical_glyph.y as f32 - placement.top as f32 + run.line_y);
				let size = Vec2::new(placement.width as f32, placement.height as f32);
				let bounds = Aabb2::new(pos, pos + size);
				ctx.painter.rect_outline(bounds, Color::grey_a(0.5, 0.1));
			}
		}

		let text_color = ct::Color::rgb(0xdd, 0x55, 0xdd);
		state.buffer.draw(font_system, &mut ctx.text_state.swash_cache, text_color, |x, y, w, h, color| {
			let pos = Vec2::new(x as f32, y as f32) + start_pos;
			let size = Vec2::new(w as f32, h as f32);
			let rect = Aabb2::new(pos, pos + size);

			ctx.painter.rect(rect, Color::from(color.as_rgba()).to_linear());
		});

		// let mut text_state = self.text_state.borrow_mut();
		// let text_state = &mut *text_state;

		// let metrics = ct::Metrics::new(24.0, 32.0);
		// let mut buffer = ct::Buffer::new(&mut text_state.font_system, metrics);
		// {
		// 	let mut buffer = buffer.borrow_with(&mut text_state.font_system);
		// 	buffer.set_size(500.0, 80.0);

		// 	let attrs = ct::Attrs::new();

		// 	buffer.set_text("Hello, Rust! 🦀 🐄🦐🅱 I'm emoting العربية ", attrs, ct::Shaping::Advanced);
		// 	buffer.shape_until_scroll(true);
		// }

		// for run in buffer.layout_runs() {
		// 	for glyph in run.glyphs.iter() {
		// 		let physical_glyph = glyph.physical((0., 0.), 1.0);

		// 		let glyph_color = match glyph.color_opt {
		// 			Some(some) => some,
		// 			None => ct::Color::rgb(0xFF, 0x55, 0xFF),
		// 		};

		// 		let Some(commands) = text_state.swash_cache.get_outline_commands(&mut text_state.font_system, physical_glyph.cache_key)
		// 		else { continue };

		// 		let to_point = |[x, y]: [f32; 2]| {
		// 			lyon::math::Point::new(x + physical_glyph.x as f32 + 100.0, -y + physical_glyph.y as f32 + 300.0)
		// 		};

		// 		let mut builder = lyon::path::Path::builder().with_svg();

		// 		for command in commands {
		// 			match *command {
		// 				ct::Command::MoveTo(p) => { builder.move_to(to_point(p.into())); }
		// 				ct::Command::LineTo(p) => { builder.line_to(to_point(p.into())); }
		// 				ct::Command::CurveTo(c1, c2, to) => { builder.cubic_bezier_to(to_point(c1.into()), to_point(c2.into()), to_point(to.into())); }
		// 				ct::Command::QuadTo(c, to) => { builder.quadratic_bezier_to(to_point(c.into()), to_point(to.into())); }
		// 				ct::Command::Close => { builder.close(); }
		// 			}
		// 		}

		// 		painter.fill_path(&builder.build(), Color::from(glyph_color.as_rgba()).to_linear());
		// 	}
		// }
	}
}

