use super::*;

use std::fmt::Debug;


#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct WidgetId(pub(super) u64);


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WidgetLifecycleEvent {
	Created,
	Updated,
	Destroyed,
}

pub struct LifecycleContext<'a> {
	pub event: WidgetLifecycleEvent,
	
	pub widget_id: WidgetId,
	pub state: &'a mut StateBox,
	pub text_atlas: &'a mut super::TextAtlas,
	pub input: &'a Input,

	pub should_redraw: &'a Cell<bool>,
}

impl LifecycleContext<'_> {
	pub fn trigger_redraw(&self) {
		self.should_redraw.set(true);
	}
}


pub struct ConfigureContext<'a> {
	pub constraints: &'a mut LayoutConstraints,
	pub constraint_map: &'a LayoutConstraintMap,
	pub children: &'a [WidgetId],

	pub input: &'a mut InputBehaviour,
	pub style: &'a mut WidgetStyle,

	pub widget_id: WidgetId,
	pub state: &'a mut StateBox,
	pub text_atlas: &'a mut super::TextAtlas,
}

pub struct DrawContext<'a> {
	pub painter: &'a mut Painter,
	pub layout: &'a Layout,

	// TODO(pat.m): these should all be combined into a style context
	pub style: &'a WidgetStyle,
	pub app_style: &'a AppStyle,
	pub text_color: WidgetColorRole,

	pub widget_id: WidgetId,
	pub state: &'a mut StateBox,
	pub text_atlas: &'a mut super::TextAtlas,
	pub input: &'a Input,
}


pub trait Widget : AsAny + Debug {
	fn lifecycle(&mut self, _: LifecycleContext<'_>) {}
	fn configure(&self, _: ConfigureContext<'_>) {}
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



pub trait StatefulWidget : Widget {
	type State: 'static;

	fn get_state<'s>(&self, state_box: &'s mut StateBox) -> &'s mut Self::State {
		state_box.get()
	}

	fn get_state_or_default<'s>(&self, state_box: &'s mut StateBox) -> &'s mut Self::State
		where Self::State: Default
	{
		state_box.get_or_default()
	}

	fn get_state_or_else<'s, F>(&self, state_box: &'s mut StateBox, create: F) -> &'s mut Self::State
		where F: FnOnce() -> Self::State
	{
		state_box.get_or_else(create)
	}
}