// use crate::prelude::*;
use crate::ui::*;

use std::fmt::{self, Debug};

pub mod button;
pub mod checkbox;
pub mod slider;
pub mod spring;
pub mod text;

pub use button::*;
pub use checkbox::*;
pub use slider::*;
pub use spring::*;
pub use text::*;



impl Ui<'_> {
	pub fn dummy(&self) -> WidgetRef<'_, ()> {
		self.add_widget(())
	}
}



impl Widget for () {
	fn configure(&self, ctx: ConfigureContext<'_>) {
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
	fn configure(&self, ctx: ConfigureContext<'_>) {
		let ConfigureContext { constraints, .. } = ctx;

		constraints.layout_axis.set_default(self.axis);
		constraints.padding.set_default(8.0);

		constraints.horizontal_size_policy.set_default(SizingBehaviour::FIXED);
		constraints.vertical_size_policy.set_default(SizingBehaviour::FIXED);
	}
}



#[derive(Debug)]
pub struct FrameWidget<I: Widget> {
	pub inner: I,
}

impl<W: Widget> FrameWidget<W> {
	pub fn new(inner: W) -> Self {
		FrameWidget {
			inner,
		}
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
	fn configure(&self, ctx: ConfigureContext<'_>) {
		if ctx.style.fill.is_none() {
			ctx.style.fill = Some(WidgetColorRole::SurfaceContainer.into());
		}

		self.inner.configure(ctx);
	}

	fn draw(&self, ctx: DrawContext<'_>) {
		self.inner.draw(ctx);
	}
}

