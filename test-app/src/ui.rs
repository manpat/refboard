use crate::prelude::*;

pub mod widget;
pub mod widget_ref;
pub mod widgets;
pub mod style;
pub mod text;
pub mod layout;
pub mod hierarchy;
pub mod input;
pub mod viewport;

pub use widget::*;
pub use widgets::*;
pub use widget_ref::*;
pub use style::*;
pub use text::*;
pub use layout::*;
pub use hierarchy::*;
pub use viewport::*;
pub use input::*;

use std::any::{TypeId, Any};
use std::marker::PhantomData;


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

			persistent_state: PersistentState::default(),
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
					behaviour: widget_state.input_behaviour,
				});

				// Reset for next frame
				// TODO(pat.m): yuck
				widget_state.input_behaviour = InputBehaviour::empty();
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


#[derive(Debug, Default)]
pub struct StateBox(Option<Box<dyn Any>>);

impl StateBox {
	pub fn has<T: 'static>(&self) -> bool {
		match &self.0 {
			Some(value) => (*value).is::<T>(),
			None => false,
		}
	}

	pub fn set<T>(&mut self, value: T)
		where T: 'static
	{
		self.0 = Some(Box::new(value));
	}

	pub fn get_or_default<T>(&mut self) -> &mut T
		where T: Default + 'static
	{
		// If we have a value but the type is wrong, reset it to default
		if !self.has::<T>() {
			self.0 = Some(Box::new(T::default()));
		}

		// SAFETY: We're calling set above, so we know that the Option is populated and with what type.
		unsafe {
			// TODO(pat.m): this could use downcast_mut_unchecked once stable
			self.0.as_mut()
				.and_then(|value| value.downcast_mut::<T>())
				.unwrap_unchecked()
		}
	}

	pub fn get<T>(&mut self) -> &mut T
		where T: 'static
	{
		assert!(self.has::<T>());

		unsafe {
			// TODO(pat.m): this could use downcast_mut_unchecked once stable
			self.0.as_mut()
				.and_then(|value| value.downcast_mut::<T>())
				.unwrap_unchecked()
		}
	}
}


#[derive(Debug)]
struct WidgetBox {
	widget: Box<dyn Widget>,
	state: StateBox,

	// TODO(pat.m): this shouldn't really be retained, just not sure where to put it
	input_behaviour: InputBehaviour,
}


#[derive(Debug, Default)]
pub struct PersistentState {
	widgets: RefCell<HashMap<WidgetId, WidgetBox>>,
	hierarchy: RefCell<Hierarchy>,
}


#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct WidgetId(u64);



#[derive(Debug)]
pub struct Ui<'ps> {
	stack: RefCell<Vec<WidgetId>>,
	widget_constraints: RefCell<LayoutConstraintMap>,

	persistent_state: &'ps PersistentState,
	pub text_state: &'ps RefCell<TextState>,
	pub input: &'ps Input,
}

impl<'ps> Ui<'ps> {
	fn new(persistent_state: &'ps PersistentState, input: &'ps Input, text_state: &'ps RefCell<TextState>) -> Ui<'ps> {
		Ui {
			stack: Default::default(),
			widget_constraints: LayoutConstraintMap::default().into(),

			persistent_state,
			text_state,
			input,
		}
	}

	fn layout(&mut self, available_bounds: Aabb2, widget_layouts: &mut LayoutMap) {
		let hierarchy = self.persistent_state.hierarchy.borrow();
		let mut widgets = self.persistent_state.widgets.borrow_mut();
		let mut widget_constraints = self.widget_constraints.borrow_mut();
		let mut text_state = self.text_state.borrow_mut();
		let text_state = &mut *text_state;

		let num_widgets = widgets.len();
		widget_constraints.reserve(num_widgets);

		// bottom up request size hints/policies and content sizes if appropriate
		hierarchy.visit_leaves_first(None, |widget_id| {
			let mut constraints = widget_constraints.get(&widget_id).cloned().unwrap_or_default();
			let children = hierarchy.children(widget_id);

			let widget_state = widgets.get_mut(&widget_id).unwrap();

			widget_state.widget.configure(ConfigureContext {
				constraints: &mut constraints,
				children,
				constraint_map: &mut widget_constraints,

				// NOTE: We're relying on this being reset in persist_input_bounds
				input_behaviour: &mut widget_state.input_behaviour,

				state: &mut widget_state.state,
				text_state,
				widget_id,
			});

			widget_constraints.insert(widget_id, constraints);
		});

		widget_layouts.clear();
		widget_layouts.reserve(num_widgets);

		for &widget_id in hierarchy.root_node.children.iter() {
			layout_children_linear(available_bounds, Axis::Horizontal, Align::Start, &[widget_id], &widget_constraints, widget_layouts);
		}

		// top down resolve layouts and assign rects
		hierarchy.visit_breadth_first(None, |widget_id, children| {
			// this widget should already be laid out or root
			let content_bounds = widget_layouts[&widget_id].content_bounds;
			let constraints = &widget_constraints[&widget_id];
			let main_axis = constraints.layout_axis.get();
			let content_alignment = constraints.content_alignment.get();

			// TODO(pat.m): layout mode?
			layout_children_linear(content_bounds, main_axis, content_alignment, children, &widget_constraints, widget_layouts);
		});

		// Calculate clip rects
		hierarchy.visit_breadth_first_with_parent_context(None, |widget_id, parent_clip| {
			let layout = widget_layouts.get_mut(&widget_id).unwrap();
			layout.clip_rect = parent_clip;

			let clip_rect = match parent_clip {
				Some(parent_clip) => clip_rects(&layout.box_bounds, &parent_clip),
				None => layout.box_bounds,
			};

			Some(clip_rect)
		});

		// TODO(pat.m): why is this not on Aabb2
		fn clip_rects(lhs: &Aabb2, rhs: &Aabb2) -> Aabb2 {
			Aabb2 {
				min: Vec2::new(lhs.min.x.max(rhs.min.x), lhs.min.y.max(rhs.min.y)),
				max: Vec2::new(lhs.max.x.min(rhs.max.x), lhs.max.y.min(rhs.max.y)),
			}
		}
	}

	fn draw(&mut self, painter: &mut Painter, widget_layouts: &LayoutMap) {
		let mut widgets = self.persistent_state.widgets.borrow_mut();
		let hierarchy = self.persistent_state.hierarchy.borrow();

		let mut text_state = self.text_state.borrow_mut();
		let text_state = &mut *text_state;

		// draw from root to leaves
		hierarchy.visit_breadth_first(None, |widget_id, _| {
			let layout = &widget_layouts[&widget_id];
			let widget_state = widgets.get_mut(&widget_id).unwrap();

			painter.set_clip_rect(layout.clip_rect);
			widget_state.widget.draw(DrawContext {
				painter,
				layout,
				text_state,

				state: &mut widget_state.state,
				input: self.input,
				widget_id,
			});
		});

		// Visualise clip rects
		// TODO(pat.m): make this a debug setting
		if false {
			painter.set_clip_rect(None);
			painter.set_color(Color::white());

			hierarchy.visit_breadth_first(None, |widget_id, _| {
				if let Some(clip) = widget_layouts[&widget_id].clip_rect {
					painter.rect_outline(clip);
				}
			});
		}
	}

	fn calc_min_size(&self) -> Vec2 {
		let widget_constraints = self.widget_constraints.borrow();
		let hierarchy = self.persistent_state.hierarchy.borrow();

		let mut min_width = 10.0;
		let mut min_height = 10.0;

		for widget_id in hierarchy.children(None) {
			let constraints = &widget_constraints[widget_id];
			min_width = constraints.min_width.get().max(min_width);
			min_height = constraints.min_height.get().max(min_height);
		}

		Vec2::new(min_width, min_height)
	}
}


impl Ui<'_> {
	pub fn parent_id(&self) -> Option<WidgetId> {
		self.stack.borrow().last().copied()
	}

	pub fn add_widget<T>(&self, widget: T) -> WidgetRef<'_, T>
		where T: Widget + 'static
	{
		self.add_widget_to(widget, self.parent_id())
	}

	pub fn add_widget_to<T>(&self, widget: T, parent_id: impl Into<Option<WidgetId>>) -> WidgetRef<'_, T>
		where T: Widget
	{
		let mut hierarchy = self.persistent_state.hierarchy.borrow_mut();
		let mut widgets = self.persistent_state.widgets.borrow_mut();
		let mut text_state = self.text_state.borrow_mut();
		let text_state = &mut *text_state;

		let parent_id = parent_id.into();
		let type_id = TypeId::of::<T>();

		// TODO(pat.m): make id fragment overrideable
		let NodeUpdateResult {
			widget_id,
			status,
		} = hierarchy.add_or_update(WidgetIdFragment::TypedOrdered(type_id), parent_id);

		match status {
			NodeUpdateStatus::Added => {
				let mut widget_box = WidgetBox {
					widget: Box::new(widget),
					state: StateBox(None),
					input_behaviour: InputBehaviour::empty(),
				};

				widget_box.widget.lifecycle(LifecycleContext {
					event: WidgetLifecycleEvent::Created,
					state: &mut widget_box.state,
					text_state,
					input: self.input,
					widget_id,
				});

				widgets.insert(widget_id, widget_box);
			}

			NodeUpdateStatus::Update => {
				let widget_box = widgets.get_mut(&widget_id).unwrap();

				// Reuse allocation if we can
				if let Some(typed_widget) = widget_box.widget.as_widget_mut::<T>() {
					*typed_widget = widget;
				} else {
					// Otherwise recreate storage
					widget_box.widget = Box::new(widget);
					// TODO(pat.m): should this still be an update event?
				}

				widget_box.widget.lifecycle(LifecycleContext {
					event: WidgetLifecycleEvent::Updated,
					state: &mut widget_box.state,
					text_state,
					input: self.input,
					widget_id,
				});
			}
		}

		WidgetRef { widget_id, ui: self, phantom: PhantomData }
	}
}


/// Layouts
impl Ui<'_> {
	pub fn push_layout(&self, widget_id: impl Into<WidgetId>) {
		self.stack.borrow_mut().push(widget_id.into());
	}

	pub fn pop_layout(&self) {
		self.stack.borrow_mut().pop().expect("Parent stack empty!");
	}

	pub fn with_parent(&self, layout_id: impl Into<WidgetId>, f: impl FnOnce()) {
		self.push_layout(layout_id);
		f();
		self.pop_layout();
	}

	pub fn with_parent_widget<W>(&self, layout_widget: W, f: impl FnOnce()) -> WidgetRef<'_, W>
		where W: Widget
	{
		let layout = self.add_widget(layout_widget);
		self.with_parent(&layout, f);
		layout
	}

	pub fn with_horizontal_layout(&self, f: impl FnOnce()) -> WidgetRef<'_, BoxLayout> {
		self.with_parent_widget(BoxLayout::horizontal(), f)
	}

	pub fn with_vertical_layout(&self, f: impl FnOnce()) -> WidgetRef<'_, BoxLayout> {
		self.with_parent_widget(BoxLayout::vertical(), f)
	}

	pub fn with_horizontal_frame(&self, f: impl FnOnce()) -> WidgetRef<'_, FrameWidget<BoxLayout>> {
		self.with_parent_widget(FrameWidget::horizontal(), f)
	}

	pub fn with_vertical_frame(&self, f: impl FnOnce()) -> WidgetRef<'_, FrameWidget<BoxLayout>> {
		self.with_parent_widget(FrameWidget::vertical(), f)
	}
}