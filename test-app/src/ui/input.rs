use crate::prelude::*;

use winit::event::{WindowEvent, KeyEvent, ElementState, MouseButton as WinitMouseButton};
use winit::dpi::PhysicalPosition;

pub use winit::window::ResizeDirection;


#[derive(Default, Debug)]
pub struct Input {
	pub cursor_pos: Option<Vec2>,

	pub events_received_this_frame: bool,

	pub hovered_widget: Option<ui::WidgetId>,
	pub active_widget: Option<ui::WidgetId>,
	pub focus_widget: Option<ui::WidgetId>,

	pub registered_widgets: HashMap<ui::WidgetId, RegisteredWidget>,
	pub keyboard_input: Vec<KeyboardEvent>,

	// The state of each mouse button as it is after all mouse events are processed.
	button_states: [MouseButtonState; 5],

	viewport: ui::Viewport,
	timestamp: Wrapping<u32>,
}

impl Input {
	pub fn reset(&mut self) {
		self.keyboard_input.clear();
		self.events_received_this_frame = false;
		self.timestamp += 1;
	}

	pub fn set_viewport(&mut self, viewport: ui::Viewport) {
		self.viewport = viewport;
	}

	#[instrument(skip_all)]
	pub fn send_event(&mut self, event: WindowEvent) -> SendEventResponse {
		self.events_received_this_frame = true;

		match event {
			WindowEvent::CursorMoved { position, .. } => {
				let PhysicalPosition {x, y} = position.cast();
				let cursor_pos = self.viewport.physical_to_view() * Vec2::new(x, y);
				self.cursor_pos = Some(cursor_pos);

				// TODO(pat.m): queue mouse move events so we don't lose precision
				// maybe this can be opt in?
			}

			WindowEvent::CursorLeft { .. } => {
				self.cursor_pos = None;
				self.hovered_widget = None;
			}

			WindowEvent::MouseInput { state: ElementState::Released, button, .. } => {
				if let Some(button) = MouseButton::try_from_winit(button) {
					self.button_state_mut(button).up_timestamp = self.timestamp.0;
					self.active_widget = None;
				}
			}

			WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
				let Some(button) = MouseButton::try_from_winit(button) else {
					return SendEventResponse::None
				};

				// TODO(pat.m): shouldn't just use hovered_widget, but should
				// pick first widget in hover stack that handles event
				if let Some(widget_id) = self.hovered_widget
				&& let Some(reg) = self.registered_widgets.get(&widget_id)
				{
					// If we hit a drag zone, then we _don't_ want to forward events to the rest of the ui
					if reg.behaviour.contains(InputBehaviour::WINDOW_DRAG_ZONE) {
						return SendEventResponse::DragWindow
					} else if reg.behaviour.contains(InputBehaviour::WINDOW_DRAG_RESIZE_ZONE) {
						return SendEventResponse::DragResizeWindow(ResizeDirection::SouthEast)
					}
				}

				// No mouse downs without a position
				if let Some(cursor_pos) = self.cursor_pos {
					self.button_state_mut(button).down_timestamp = self.timestamp.0;
					self.button_state_mut(button).last_press_position = cursor_pos;
				}
			}

			WindowEvent::KeyboardInput { event, .. } => {
				if let Some(text) = event.text {
					self.keyboard_input.extend(text.chars().map(KeyboardEvent::Character));
				}

				// TODO(pat.m): key chords, modifiers
			}

			_ => {}
		}

		SendEventResponse::None
	}

	#[instrument(skip_all)]
	pub fn process_events(&mut self, hierarchy: &ui::Hierarchy) {
		if let Some(cursor_pos) = self.cursor_pos {
			// TODO(pat.m): instead of just storing the last hovered widget, store a 'stack' of hovered widgets
			hierarchy.visit_breadth_first(|widget_id, _| {
				if let Some(widget_info) = self.registered_widgets.get(&widget_id)
					&& widget_info.bounds.contains_point(cursor_pos)
				{
					self.hovered_widget = Some(widget_id);
				}
			});
		}

		// TODO(pat.m): this is completely bogus
		if self.hovered_widget.is_some() {
			if self.button_states.iter()
				.any(|s| s.up_timestamp == self.timestamp.0)
			{
				self.focus_widget = self.hovered_widget;
			}

			if self.is_any_mouse_down() && self.active_widget.is_none() {
				self.active_widget = self.hovered_widget;
			}
		}

		// TODO(pat.m): from the input behaviour of each widget in the hovered widget stack, calculate the target of 
		// any mouse click/keyboard events.

		// TODO(pat.m): if a mouse up event occurs, set focus to whatever the top most focusable widget is
	}

	#[instrument(skip_all)]
	pub fn register_handlers(&mut self, hierarchy: &ui::Hierarchy,
		widgets: &HashMap<ui::WidgetId, ui::WidgetBox>, layouts: &ui::LayoutMap)
	{
		self.registered_widgets.clear();

		hierarchy.visit_breadth_first_with_cf(|widget_id, _| {
			let widget_state = widgets.get(&widget_id).unwrap();
			let behaviour = widget_state.config.input;

			let blocks_input_to_children = behaviour.contains(InputBehaviour::OPAQUE);
			let receives_input = !behaviour.contains(InputBehaviour::TRANSPARENT);

			if receives_input {
				let bounds = layouts[&widget_id].box_bounds;
				self.registered_widgets.insert(widget_id, RegisteredWidget {bounds, behaviour});
			}

			!blocks_input_to_children
		});
	}

	fn button_state(&self, button: MouseButton) -> &MouseButtonState {
		&self.button_states[button as usize]
	}

	fn button_state_mut(&mut self, button: MouseButton) -> &mut MouseButtonState {
		&mut self.button_states[button as usize]
	}
}

#[derive(Debug, Copy, Clone)]
pub enum SendEventResponse {
	None,
	DragWindow,
	DragResizeWindow(ResizeDirection),
}


impl Input {
	/// Get the last state of a mouse button
	pub fn is_mouse_down(&self, button: MouseButton) -> bool {
		self.button_state(button).is_down()
	}

	pub fn is_any_mouse_down(&self) -> bool {
		self.button_states.iter()
			.any(|state| state.is_down())
	}

	/// Returns whether the last event for a mouse button was a down event
	pub fn was_mouse_pressed(&self, button: MouseButton) -> bool {
		self.button_state(button).down_timestamp == self.timestamp.0
	}

	/// Returns whether the last event for a mouse button was an up event
	pub fn was_mouse_released(&self, button: MouseButton) -> bool {
		self.button_state(button).up_timestamp == self.timestamp.0
	}

	pub fn mouse_drag_delta(&self, button: MouseButton) -> Option<Vec2> {
		let state = self.button_state(button);
		if state.is_down() {
			Some(self.cursor_pos? - state.last_press_position)
		} else {
			None
		}
	}
}


#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u8)]
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


bitflags! {
	#[derive(Debug, Copy, Clone, Eq, PartialEq)]
	pub struct InputBehaviour : u32 {
		// TODO(pat.m): handles left/right/etc mouse events
		// TODO(pat.m): handles scroll events?
		// TODO(pat.m): handles key events/ime?
		// TODO(pat.m): draggable? focusable?
		// TODO(pat.m): capture on mouse down? maybe this should be implicit
		// TODO(pat.m): ignores clipping?

		/// Do not accept any input events.
		const TRANSPARENT = 1<<0;

		/// Do not forward input events to children
		const OPAQUE = 1<<1;

		const WINDOW_DRAG_ZONE = 1<<10;
		const WINDOW_DRAG_RESIZE_ZONE = 1<<11;
	}
}

impl Default for InputBehaviour {
	fn default() -> Self { Self::empty() }
}

#[derive(Debug)]
pub struct RegisteredWidget {
	pub bounds: Aabb2,
	pub behaviour: InputBehaviour,
}

#[derive(Debug, Default)]
pub struct MouseButtonState {
	pub last_press_position: Vec2,
	pub down_timestamp: u32,
	pub up_timestamp: u32,
}

impl MouseButtonState {
	pub fn is_down(&self) -> bool {
		self.down_timestamp > self.up_timestamp
	}
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum KeyboardEvent {
	Character(char),
}