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
}



impl Widget for () {
	fn constrain(&self, ctx: ConstraintContext<'_>) {
		ctx.constraints.margin.set_default(4.0);
		ctx.constraints.padding.set_default(8.0);

		ctx.constraints.horizontal_size_policy.set_default(SizingBehaviour::FLEXIBLE);
	}

	fn draw(&self, painter: &mut Painter, layout: &Layout, _: &mut StateBox) {
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
	fn draw(&self, painter: &mut Painter, layout: &Layout, _: &mut StateBox) {
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

	fn draw(&self, painter: &mut Painter, layout: &Layout, state: &mut StateBox) {
		let rounding = 4.0;

		painter.rounded_rect(layout.box_bounds, rounding, self.background_color);
		painter.rounded_rect_outline(layout.box_bounds, rounding, self.outline_color);

		self.inner.draw(painter, layout, state);
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

	fn draw(&self, painter: &mut Painter, layout: &Layout, _state: &mut StateBox) {
		let rounding = 4.0;

		painter.rounded_rect(layout.box_bounds, rounding, Color::grey(0.3));
		painter.rounded_rect_outline(layout.box_bounds, rounding, Color::grey(0.5));
	}
}

