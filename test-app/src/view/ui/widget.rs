use crate::prelude::*;
use super::{WidgetId, Ui, StateBox, Layout, LayoutConstraints, LayoutConstraintMap};

use std::cell::{RefMut};
use std::fmt::Debug;
use std::marker::PhantomData;


pub struct ConstraintContext<'a> {
	pub constraints: &'a mut LayoutConstraints,
	pub constraint_map: &'a LayoutConstraintMap,
	pub children: &'a [WidgetId],

	pub state: &'a mut StateBox,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WidgetLifecycleEvent {
	Created,
	Updated,
	Destroyed,
}

pub trait Widget : AsAny + Debug {
	fn constrain(&self, _: ConstraintContext<'_>) {}
	fn draw(&self, _painter: &mut Painter, _layout: &Layout, _: &mut StateBox) {}

	fn lifecycle(&mut self, _event: WidgetLifecycleEvent, _state: &mut StateBox) {
		if _event != WidgetLifecycleEvent::Updated {
			let type_name = (*self).type_name();
			println!("widget lifecycle {_event:?}: '{type_name}' ------ {_state:?}");
		}
	}
}

impl dyn Widget {
	pub fn is_widget<T: Widget>(&self) -> bool {
		(*self).as_any().is::<T>()
	}

	pub fn as_widget<T: Widget>(&self) -> Option<&T> {
		(*self).as_any()
			.downcast_ref::<T>()
	}

	pub fn as_widget_mut<T: Widget>(&mut self) -> Option<&mut T> {
		(*self).as_any_mut()
			.downcast_mut::<T>()
	}
}


#[derive(Debug, Copy, Clone)]
pub struct WidgetRef<'ui, T> {
	pub widget_id: WidgetId,
	pub ui: &'ui Ui<'ui>,

	pub phantom: PhantomData<&'ui T>,
}

impl<'ui, T> WidgetRef<'ui, T> {
	pub fn with_constraints(self, mutate: impl FnOnce(&mut LayoutConstraints)) -> Self {
		mutate(&mut *self.constraints());
		self
	}

	pub fn with_widget(self, mutate: impl FnOnce(&mut T, &mut StateBox)) -> Self
		where T: Widget
	{
		let (mut widget, mut state) = self.widget_box();
		mutate(widget.as_widget_mut().unwrap(), &mut *state);
		self
	}

	pub fn widget(&self) -> RefMut<'ui, T>
		where T: Widget
	{
		RefMut::map(self.widget_box().0, |w| w.as_widget_mut().unwrap())
	}

	pub fn state(&self) -> RefMut<'ui, StateBox> {
		self.widget_box().1
	}

	pub fn state_as<S>(&self) -> RefMut<'ui, S>
		where S: Default + 'static
	{
		RefMut::map(self.state(), |s| s.get())
	}

	pub fn constraints(&self) -> RefMut<'ui, LayoutConstraints> {
		RefMut::map(
			self.ui.widget_constraints.borrow_mut(),
			|wcs| wcs.entry(self.widget_id).or_default()
		)
	}

	fn widget_box(&self) -> (RefMut<'ui, dyn Widget>, RefMut<'ui, StateBox>) {
		RefMut::map_split(
			self.ui.persistent_state.widgets.borrow_mut(),
			|widgets| {
				let widget_box = widgets.get_mut(&self.widget_id).unwrap();
				(&mut *widget_box.widget, &mut widget_box.state)
			}
		)
	}

	pub fn is_hovered(&self) -> bool {
		self.ui.persistent_state.hovered_widget.get() == Some(self.widget_id)
	}

	pub fn is_clicked(&self) -> bool {
		self.is_hovered() && self.ui.input.click_received
	}

	// TODO(pat.m): is focussed
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