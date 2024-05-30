use crate::prelude::*;

use winit::event::{WindowEvent, ElementState, MouseButton};
use winit::dpi::PhysicalPosition;

#[derive(Default, Debug)]
pub struct Input {
	raw_cursor_pos: Option<Vec2>,

	pub cursor_pos_view: Option<Vec2>,
	pub click_received: bool,

	pub events_received_this_frame: bool,

	pub hovered_widget: Option<ui::WidgetId>,
}

impl Input {
	pub fn reset(&mut self) {
		self.cursor_pos_view = None;
		self.click_received = false;
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

			WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
				// TODO(pat.m): better
				self.click_received = true;
			}

			_ => {}
		}
	}
}
