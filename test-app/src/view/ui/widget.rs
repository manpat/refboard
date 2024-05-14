use crate::prelude::*;
use super::{WidgetId, Layout, LayoutConstraints, LayoutConstraintMap};

use std::any::Any;
use std::fmt::Debug;


pub struct ConstraintContext<'a> {
	pub constraints: &'a mut LayoutConstraints,
	pub children: &'a [WidgetId],
	pub constraint_map: &'a LayoutConstraintMap,
}


pub trait Widget : Any + Debug {
	fn as_any(&self) -> &dyn Any
		where Self: Sized
	{
		self as &dyn Any
	}

	fn as_any_mut(&mut self) -> &mut dyn Any
		where Self: Sized
	{
		self as &mut dyn Any
	}

	fn calculate_constraints(&self, _: ConstraintContext<'_>) {}

	fn draw(&self, painter: &mut Painter, layout: &Layout) {
		let widget_color = Color::grey_a(0.5, 0.1);
		painter.rounded_rect_outline(layout.box_bounds, 8.0, widget_color);
	}
}



#[derive(Debug, Copy, Clone)]
pub struct WidgetRef<'ui> {
	pub widget_id: WidgetId,
	pub constraints_map: &'ui RefCell<LayoutConstraintMap>,
}

impl<'ui> WidgetRef<'ui> {
	pub fn set_constraints(self, mutate: impl FnOnce(&mut LayoutConstraints)) -> Self {
		let mut lcs = self.constraints_map.borrow_mut();
		mutate(lcs.entry(self.widget_id).unwrap().or_default());
		self
	}
}

impl<'ui> From<&'_ WidgetRef<'ui>> for WidgetId {
	fn from(o: &'_ WidgetRef<'ui>) -> WidgetId {
		o.widget_id
	}
}

impl<'ui> From<&'_ WidgetRef<'ui>> for Option<WidgetId> {
	fn from(o: &'_ WidgetRef<'ui>) -> Option<WidgetId> {
		Some(o.widget_id)
	}
}

impl<'ui> From<WidgetRef<'ui>> for WidgetId {
	fn from(o: WidgetRef<'ui>) -> WidgetId {
		o.widget_id
	}
}

impl<'ui> From<WidgetRef<'ui>> for Option<WidgetId> {
	fn from(o: WidgetRef<'ui>) -> Option<WidgetId> {
		Some(o.widget_id)
	}
}