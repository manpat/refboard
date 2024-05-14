use crate::prelude::*;
use crate::view::ui::*;

use std::fmt::{self, Debug};



impl Widget for () {
	fn calculate_constraints(&self, ctx: ConstraintContext<'_>) {
		ctx.constraints.margin.set_default(4.0);
		ctx.constraints.padding.set_default(8.0);

		ctx.constraints.horizontal_size_policy.set_default(SizingBehaviour::FLEXIBLE);
	}

	fn draw(&self, painter: &mut Painter, layout: &Layout) {
		let widget_color = [0.5; 3];
		painter.rounded_rect_outline(layout.box_bounds, 8.0, widget_color);
		painter.line(layout.box_bounds.min, layout.box_bounds.max, widget_color);
		painter.line(layout.box_bounds.min_max_corner(), layout.box_bounds.max_min_corner(), widget_color);

		painter.rounded_rect_outline(layout.content_bounds, 8.0, [0.5, 1.0, 0.5, 0.5]);
	}
}




pub struct DrawFnWidget<F>(pub F)
	where F: Fn(&mut Painter, &Layout) + 'static;

impl<F> Widget for DrawFnWidget<F>
	where F: Fn(&mut Painter, &Layout) + 'static
{
	fn draw(&self, painter: &mut Painter, layout: &Layout) {
		(self.0)(painter, layout);
	}
}

impl<F> Debug for DrawFnWidget<F>
	where F: Fn(&mut Painter, &Layout) + 'static
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
	fn calculate_constraints(&self, ctx: ConstraintContext<'_>) {
		let ConstraintContext { constraints, children, constraint_map } = ctx;

		constraints.layout_axis.set_default(self.axis);

		let main_axis = constraints.layout_axis.get();
		let cross_axis = main_axis.opposite();

		let mut total_main_min_length = 0.0f32;
		let mut total_cross_min_length = 0.0f32;

		let mut total_main_preferred_length = 0.0f32;
		let mut total_cross_preferred_length = 0.0f32;

		for &child in children {
			let child_constraints = &constraint_map[child];

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

	outline_color: Color,
	background_color: Color,
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
	fn calculate_constraints(&self, ctx: ConstraintContext<'_>) {
		self.inner.calculate_constraints(ctx);
	}

	fn draw(&self, painter: &mut Painter, layout: &Layout) {
		let rounding = 4.0;

		painter.rounded_rect(layout.box_bounds, rounding, self.background_color);
		painter.rounded_rect_outline(layout.box_bounds, rounding, self.outline_color);

		self.inner.draw(painter, layout);
	}
}
