use crate::prelude::*;

use winit::event::{WindowEvent, ElementState, MouseButton as WinitMouseButton};
use winit::dpi::PhysicalPosition;


#[derive(Default, Debug)]
pub struct Input {
	raw_cursor_pos: Option<Vec2>,

	pub cursor_pos_view: Option<Vec2>,

	pub events_received_this_frame: bool,

	pub hovered_widget: Option<ui::WidgetId>,

	// All events that happened this frame.
	events: Vec<InputEvent>,

	// The state of each mouse button as it is after all mouse events are processed.
	mouse_states: HashMap<MouseButton, bool>,
}

impl Input {
	pub fn reset(&mut self) {
		self.events.clear();

		self.cursor_pos_view = None;
		self.events_received_this_frame = false;
		self.hovered_widget = None;
	}

	pub fn process_events(&mut self, viewport: &ui::Viewport) {
		self.cursor_pos_view = self.raw_cursor_pos
			.map(|raw_pos| viewport.physical_to_view() * raw_pos);
	}

	pub fn send_event(&mut self, event: WindowEvent) {
		self.events_received_this_frame = true;

		match event {
			WindowEvent::CursorMoved { position, .. } => {
				let PhysicalPosition {x, y} = position.cast();
				self.raw_cursor_pos = Some(Vec2::new(x, y));
			}

			WindowEvent::CursorLeft { .. } => {
				self.raw_cursor_pos = None;
			}

			WindowEvent::MouseInput { state: ElementState::Released, button, .. } => {
				if let Some(button) = MouseButton::try_from_winit(button) {
					self.events.push(InputEvent::MouseUp(button));
					self.mouse_states.insert(button, false);
				}
			}

			WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
				if let Some(button) = MouseButton::try_from_winit(button) {
					self.events.push(InputEvent::MouseDown(button));
					self.mouse_states.insert(button, true);
				}

				// TODO(pat.m): immediately test interactive regions from prev frame
				// if any are registered as claiming ownership or as being window hit regions
				// inform the window immediately to avoid timing problems
			}

			_ => {}
		}
	}
}


impl Input {
	pub fn all_events(&self) -> &[InputEvent] {
		&self.events
	}

	/// Get the last state of a mouse button
	pub fn is_mouse_down(&self, button: MouseButton) -> bool {
		self.mouse_states.get(&button).copied()
			.unwrap_or(false)
	}

	/// Returns whether the last event for a mouse button was a down event
	pub fn was_mouse_pressed(&self, button: MouseButton) -> bool {
		let last_evt = self.events.iter()
			.rfind(|e| e.concerns_mouse_button(button));

		match last_evt {
			Some(InputEvent::MouseDown(_)) => true,
			_ => false,
		}
	}

	/// Returns whether the last event for a mouse button was an up event
	pub fn was_mouse_released(&self, button: MouseButton) -> bool {
		let last_evt = self.events.iter()
			.rfind(|e| e.concerns_mouse_button(button));

		match last_evt {
			Some(InputEvent::MouseUp(_)) => true,
			_ => false,
		}
	}
}


#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum MouseButton {
	Left,
	Right,
	Middle,
	Back,
	Forward,
}

impl MouseButton {
	fn try_from_winit(mb: WinitMouseButton) -> Option<Self> {
		match mb {
			WinitMouseButton::Left => Some(MouseButton::Left),
			WinitMouseButton::Right => Some(MouseButton::Right),
			WinitMouseButton::Middle => Some(MouseButton::Middle),
			WinitMouseButton::Back => Some(MouseButton::Back),
			WinitMouseButton::Forward => Some(MouseButton::Forward),
			_ => None
		}
	}
}

#[derive(Debug, Clone)]
pub enum InputEvent {
	MouseDown(MouseButton),
	MouseUp(MouseButton),

	// TODO(pat.m): character input events
	// TODO(pat.m): key events
}

impl InputEvent {
	pub fn concerns_mouse_button(&self, desired_button: MouseButton) -> bool {
		match self {
			InputEvent::MouseUp(button) => desired_button == *button,
			InputEvent::MouseDown(button) => desired_button == *button,

			_ => false
		}
	}
}