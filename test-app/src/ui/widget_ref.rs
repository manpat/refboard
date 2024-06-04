use crate::prelude::*;
use super::*;

use std::cell::RefMut;
use std::marker::PhantomData;

#[derive(Copy, Clone)]
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

	pub fn with_style(self, mutate: impl FnOnce(&mut WidgetStyle)) -> Self {
		mutate(&mut *self.style());
		self
	}

	pub fn with_input_behaviour(self, input_behaviour: InputBehaviour) -> Self
		where T: Widget
	{
		let mut widgets = self.ui.persistent_state.widgets.borrow_mut();
		widgets.get_mut(&self.widget_id).unwrap().config.input = input_behaviour;
		self
	}

	pub fn widget(&self) -> RefMut<'ui, T>
		where T: Widget
	{
		RefMut::map(self.widget_box().0, |w| w.as_widget_mut().unwrap())
	}

	pub fn any_state(&self) -> RefMut<'ui, StateBox> {
		self.widget_box().1
	}

	pub fn state_as<S>(&self) -> RefMut<'ui, S>
		where S: Default + 'static
	{
		RefMut::map(self.any_state(), |s| s.get_or_default())
	}

	pub fn constraints(&self) -> RefMut<'ui, LayoutConstraints> {
		RefMut::map(
			self.ui.widget_constraints.borrow_mut(),
			|wcs| wcs.entry(self.widget_id).or_default()
		)
	}

	pub fn style(&self) -> RefMut<'ui, WidgetStyle> {
		RefMut::map(self.config(), |c| &mut c.style)
	}

	pub fn config(&self) -> RefMut<'ui, WidgetConfiguration> {
		RefMut::map(
			self.ui.persistent_state.widgets.borrow_mut(),
			|widgets| {
				let widget_state = widgets.get_mut(&self.widget_id).unwrap();
				&mut widget_state.config
			}
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
		self.ui.input.hovered_widget == Some(self.widget_id)
	}

	pub fn is_clicked(&self) -> bool {
		self.is_hovered() && self.ui.input.was_mouse_released(ui::MouseButton::Left)
	}

	// TODO(pat.m): is focussed
}

impl<'ui, T> WidgetRef<'ui, T>
	where T: StatefulWidget
{
	pub fn state_or_default(&self) -> RefMut<'ui, T::State>
		where T::State: Default
	{
		self.state_as()
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