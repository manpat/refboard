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





#[derive(Debug)]
pub struct Text(pub String);

#[derive(Debug)]
struct TextState {
	buffer: cosmic_text::Buffer,
}

impl Widget for Text {
	fn constrain(&self, ctx: ConstraintContext<'_>) {
		ctx.constraints.min_width.set_default(100.0);
		ctx.constraints.min_height.set_default(32.0);

		ctx.constraints.horizontal_size_policy.set_default(SizingBehaviour::FIXED);
		ctx.constraints.vertical_size_policy.set_default(SizingBehaviour::FIXED);
	}

	// fn lifecycle(&mut self, event: WidgetLifecycleEvent, state: &mut StateBox)

	fn draw(&self, ctx: DrawContext<'_>) {
		ctx.painter.rect_outline(ctx.layout.box_bounds, Color::grey(0.5));


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


		// let text_color = ct::Color::rgb(0xFF, 0xFF, 0xFF);
		// buffer.draw(&mut text_state.swash_cache, text_color, |x, y, w, h, color| {
		// 	let pos = Vec2::new(x as f32 + 100.0, y as f32 + 300.0);
		// 	let size = Vec2::new(w as f32 + 1.0, h as f32 + 1.0);
		// 	let rect = Aabb2::new(pos, pos + size);

		// 	painter.rect(rect, Color::from(color.as_rgba()).to_linear());
		// });
	}
}

