use crate::prelude::*;




#[derive(Debug, Default, Clone)]
pub struct WidgetConfiguration {
	// pub layout: ui::LayoutConstraints,
	pub input: ui::InputBehaviour,
	pub style: ui::WidgetStyle,
}



#[derive(Debug, Clone, Copy, Default)]
pub struct WidgetParameter<T> {
	value: T,
	set: bool,
}

impl<T> WidgetParameter<T> {
	pub fn new(initial_value: T) -> Self {
		WidgetParameter {
			value: initial_value,
			set: false,
		}
	}

	pub fn is_set(&self) -> bool {
		self.set
	}

	pub fn set(&mut self, value: T) {
		self.value = value;
		self.set = true;
	}

	pub fn set_default(&mut self, value: T) {
		if !self.set {
			self.value = value;
		}
	}

	pub fn get(&self) -> T
		where T: Copy
	{
		self.value
	}

	pub fn get_or(&self, default: T) -> T
		where T: Copy
	{
		if self.set {
			self.value
		} else {
			default
		}
	}
}