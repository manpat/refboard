use crate::prelude::*;
use super::*;

pub struct System {
	pub viewport: Viewport,
	pub input: Input,
	pub text_state: RefCell<TextState>,
	pub min_size: Vec2,

	persistent_state: PersistentState,
	should_redraw: bool,

	widget_layouts: LayoutMap,
}

impl System {
	pub fn new() -> System {
		System {
			viewport: Viewport::default(),
			input: Input::default(),
			text_state: TextState::new().into(),
			min_size: Vec2::zero(),

			persistent_state: PersistentState::new(),
			should_redraw: true,

			widget_layouts: LayoutMap::new(),
		}
	}

	pub fn set_size(&mut self, new_size: Vec2i) {
		if self.viewport.size.to_vec2i() != new_size {
			self.should_redraw = true;
			self.viewport.size = new_size.to_vec2();
		}
	}

	pub fn prepare_next_frame(&mut self) {
		self.should_redraw = false;
		self.input.reset();
	}

	pub fn should_redraw(&self) -> bool {
		self.should_redraw || self.input.events_received_this_frame
	}

	// TODO(pat.m): could this be built around the same mechanism as std::thread::scope?
	pub fn run(&mut self, painter: &mut Painter, build_ui: impl FnOnce(&Ui<'_>)) {
		self.persistent_state.hierarchy.get_mut().new_epoch();

		self.interpret_input();

		let mut ui = Ui::new(&self.persistent_state, &self.input, &self.text_state);

		build_ui(&ui);

		self.garbage_collect();

		ui.layout(self.viewport.view_bounds(), &mut self.widget_layouts);
		ui.draw(painter, &self.widget_layouts);

		self.min_size = ui.calc_min_size();

		self.persist_input_bounds();
	}

	fn interpret_input(&mut self) {
		self.input.process_events(&self.viewport);

		// TODO(pat.m): collect input behaviour from existing widgets

		// TODO(pat.m): move this into Input::process_events
		if let Some(cursor_pos) = self.input.cursor_pos_view {
			let input_handlers = &self.input.registered_widgets;

			// TODO(pat.m): instead of just storing the last hovered widget, store a 'stack' of hovered widgets
			self.persistent_state.hierarchy.borrow()
				.visit_breadth_first(None, |widget_id, _| {
					if input_handlers[&widget_id].bounds.contains_point(cursor_pos) {
						self.input.hovered_widget = Some(widget_id);
					}
				});

			// TODO(pat.m): from the input behaviour of each widget in the hovered widget stack, calculate the target of 
			// any mouse click/keyboard events.
		}
	}

	fn persist_input_bounds(&mut self) {
		// Persist widget bounds
		let input_handlers = &mut self.input.registered_widgets;
		let widgets = self.persistent_state.widgets.get_mut();

		input_handlers.clear();

		self.persistent_state.hierarchy.borrow()
			.visit_breadth_first(None, |widget_id, _| {
				// TODO(pat.m): clipping! also we probably want some per-widget configuration of input behaviour
				let box_bounds = self.widget_layouts[&widget_id].box_bounds;
				let widget_state = widgets.get_mut(&widget_id).unwrap();

				input_handlers.insert(widget_id, RegisteredWidget {
					bounds: box_bounds,
					behaviour: widget_state.config.input,
				});
			});
	}

	fn garbage_collect(&self) {
		let mut widgets = self.persistent_state.widgets.borrow_mut();
		let mut text_state = self.text_state.borrow_mut();
		let text_state = &mut *text_state;

		self.persistent_state.hierarchy.borrow_mut()
			.collect_stale_nodes(move |widget_id| {
				if let Some(mut widget_state) = widgets.remove(&widget_id) {
					widget_state.widget.lifecycle(LifecycleContext {
						event: WidgetLifecycleEvent::Destroyed,
						state: &mut widget_state.state,
						text_state,
						input: &self.input,
						widget_id,
					});
				}
			});
	}
}
