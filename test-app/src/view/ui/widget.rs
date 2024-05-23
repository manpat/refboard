use crate::prelude::*;
use super::{WidgetId, Ui, StateBox, Layout, LayoutConstraints, LayoutConstraintMap};

use std::fmt::Debug;
use std::marker::PhantomData;


pub struct ConstraintContext<'a> {
	pub constraints: &'a mut LayoutConstraints,
	pub children: &'a [WidgetId],
	pub constraint_map: &'a LayoutConstraintMap,

	pub state: &'a mut StateBox,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WidgetLifecycleEvent {
	Created,
	Updated,
	Destroyed,
}

pub trait Widget : AsAny + Debug {
	// type State: WidgetState;
	fn constrain(&self, _: ConstraintContext<'_>) {}
	fn draw(&self, _painter: &mut Painter, _layout: &Layout, _: &mut StateBox) {}

	fn lifecycle(&mut self, _event: WidgetLifecycleEvent, _state: &mut StateBox) {
		if _event != WidgetLifecycleEvent::Updated {
			let type_name = (*self).type_name();
			println!("widget lifecycle {_event:?}: '{type_name}' ------ {_state:?}");
		}
	}
}


#[derive(Debug, Copy, Clone)]
pub struct WidgetRef<'ui, T> {
	pub widget_id: WidgetId,
	pub ui: &'ui Ui<'ui>,

	pub phantom: PhantomData<&'ui T>,
}

impl<'ui, T> WidgetRef<'ui, T> {
	pub fn set_constraints(self, mutate: impl FnOnce(&mut LayoutConstraints)) -> Self {
		let mut lcs = self.ui.layout_constraints.borrow_mut();
		mutate(lcs.entry(self.widget_id).or_default());
		self
	}

	pub fn widget<R>(&self, mutate: impl FnOnce(&mut T, &mut StateBox) -> R) -> Option<R>
		where T: Widget
	{
		let mut widgets = self.ui.persistent_state.widgets.borrow_mut();
		let widget_state = widgets.get_mut(&self.widget_id)?;
		let widget: &mut dyn Widget = &mut *widget_state.widget;

		widget.as_any_mut().downcast_mut::<T>()
			.map(|widget| mutate(widget, &mut widget_state.state))
	}

	pub fn is_hovered(&self) -> bool {
		self.ui.persistent_state.hovered_widget.get() == Some(self.widget_id)
	}

	pub fn is_clicked(&self) -> bool {
		self.is_hovered() && self.ui.click_happened
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