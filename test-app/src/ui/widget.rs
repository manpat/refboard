use crate::prelude::*;
use super::{WidgetId, Input, StateBox, Layout, LayoutConstraints, LayoutConstraintMap};

use std::fmt::Debug;


pub struct ConstraintContext<'a> {
	pub constraints: &'a mut LayoutConstraints,
	pub constraint_map: &'a LayoutConstraintMap,
	pub children: &'a [WidgetId],
	pub widget_id: WidgetId,

	pub state: &'a mut StateBox,
	pub text_state: &'a mut super::TextState,
}

pub struct DrawContext<'a> {
	pub painter: &'a mut Painter,
	pub layout: &'a Layout,
	pub widget_id: WidgetId,

	pub state: &'a mut StateBox,
	pub text_state: &'a mut super::TextState,
	pub input: &'a Input,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WidgetLifecycleEvent {
	Created,
	Updated,
	Destroyed,
}

pub struct LifecycleContext<'a> {
	pub event: WidgetLifecycleEvent,
	pub state: &'a mut StateBox,
	pub widget_id: WidgetId,
	
	pub text_state: &'a mut super::TextState,
	pub input: &'a Input,
}


pub trait Widget : AsAny + Debug {
	fn lifecycle(&mut self, _: LifecycleContext<'_>) {}
	fn constrain(&self, _: ConstraintContext<'_>) {}
	fn draw(&self, _: DrawContext<'_>) {}
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