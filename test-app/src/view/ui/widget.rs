use crate::prelude::*;
use super::{WidgetId, Ui, Layout, LayoutConstraints, LayoutConstraintMap};

use std::any::{Any, TypeId};
use std::fmt::Debug;


pub struct ConstraintContext<'a> {
	pub constraints: &'a mut LayoutConstraints,
	pub children: &'a [WidgetId],
	pub constraint_map: &'a LayoutConstraintMap,
}


pub trait AsAny : Any {
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> AsAny for T
	where T: Any
{
	fn as_any(&self) -> &dyn Any { self }
	fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub trait Widget : AsAny + Debug {
	fn calculate_constraints(&self, _: ConstraintContext<'_>) {}

	fn draw(&self, _painter: &mut Painter, _layout: &Layout) {}
}



#[derive(Debug, Copy, Clone)]
pub struct WidgetRef<'ui> {
	pub widget_id: WidgetId,
	pub ui: &'ui Ui,
}

impl<'ui> WidgetRef<'ui> {
	pub fn set_constraints(self, mutate: impl FnOnce(&mut LayoutConstraints)) -> Self {
		let mut lcs = self.ui.layout_constraints.borrow_mut();
		mutate(lcs.entry(self.widget_id).unwrap().or_default());
		self
	}

	// TODO(pat.m): W could be provided here
	pub fn widget<W, R>(&self, mutate: impl FnOnce(&mut W) -> R) -> Option<R>
		where W: Widget
	{
		let mut widgets = self.ui.widgets.borrow_mut();
		let mut widget = &mut *widgets[self.widget_id];

		widget.as_any_mut().downcast_mut::<W>()
			.map(mutate)
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