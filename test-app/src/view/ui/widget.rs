use crate::prelude::*;
use super::{WidgetId, Ui, Layout, LayoutConstraints, LayoutConstraintMap};

use std::any::{Any, TypeId};
use std::fmt::Debug;

use std::marker::PhantomData;


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
pub struct WidgetRef<'ui, T> {
	pub widget_id: WidgetId,
	pub ui: &'ui Ui,

	pub phantom: PhantomData<&'ui T>,
}

impl<'ui, T> WidgetRef<'ui, T> {
	pub fn set_constraints(self, mutate: impl FnOnce(&mut LayoutConstraints)) -> Self {
		let mut lcs = self.ui.layout_constraints.borrow_mut();
		mutate(lcs.entry(self.widget_id).unwrap().or_default());
		self
	}

	// TODO(pat.m): W could be provided here
	pub fn widget<R>(&self, mutate: impl FnOnce(&mut T) -> R) -> Option<R>
		where T: Widget
	{
		let mut widgets = self.ui.widgets.borrow_mut();
		let mut widget = &mut *widgets[self.widget_id];

		widget.as_any_mut().downcast_mut::<T>()
			.map(mutate)
	}
}

impl<'ui, T> From<&'_ WidgetRef<'ui, T>> for WidgetId {
	fn from(o: &'_ WidgetRef<'ui, T>) -> WidgetId {
		o.widget_id
	}
}

impl<'ui, T> From<&'_ WidgetRef<'ui, T>> for Option<WidgetId> {
	fn from(o: &'_ WidgetRef<'ui, T>) -> Option<WidgetId> {
		Some(o.widget_id)
	}
}

impl<'ui, T> From<WidgetRef<'ui, T>> for WidgetId {
	fn from(o: WidgetRef<'ui, T>) -> WidgetId {
		o.widget_id
	}
}

impl<'ui, T> From<WidgetRef<'ui, T>> for Option<WidgetId> {
	fn from(o: WidgetRef<'ui, T>) -> Option<WidgetId> {
		Some(o.widget_id)
	}
}