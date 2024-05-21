use crate::prelude::*;

pub mod widget;
pub mod widgets;
pub mod layout;
pub mod hierarchy;

pub use widget::*;
pub use widgets::*;
pub use layout::*;
pub use hierarchy::*;

use super::Input;

use std::any::TypeId;
use std::marker::PhantomData;
// use std::hash::{DefaultHasher, Hasher, Hash};


pub struct System {
	persistent_state: PersistentState,
}

impl System {
	pub fn new() -> System {
		System {
			persistent_state: PersistentState::default(),
		}
	}

	// TODO(pat.m): could this be built around the same mechanism as std::thread::scope?
	pub fn run(&mut self, bounds: Aabb2, painter: &mut Painter, input: &Input, build_ui: impl FnOnce(&Ui<'_>)) {
		self.persistent_state.hierarchy.get_mut().new_epoch();

		let mut ui = Ui::new(&self.persistent_state);
		ui.click_happened = input.click_received;

		// TODO(pat.m): handle input first, using cached input handlers from previous frame

		build_ui(&ui);

		let mut widgets = ui.persistent_state.widgets.borrow_mut();
		ui.persistent_state.hierarchy.borrow_mut()
			.collect_stale_nodes(move |widget_id| {
				println!("Remove {widget_id:?}!");

				if let Some(mut widget) = widgets.remove(&widget_id) {
					widget.lifecycle(WidgetLifecycleEvent::Destroyed);
				}
			});

		ui.layout(bounds);
		ui.handle_input(input);
		ui.draw(painter);
	}
}

#[derive(Debug, Default)]
pub struct PersistentState {
	widgets: RefCell<HashMap<WidgetId, Box<dyn Widget>>>,
	hierarchy: RefCell<Hierarchy>,

	hovered_widget: Cell<Option<WidgetId>>,
}


#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct WidgetId(u64);



#[derive(Debug)]
pub struct Ui<'ps> {
	stack: RefCell<Vec<WidgetId>>,
	layout_constraints: RefCell<LayoutConstraintMap>,

	widget_layouts: LayoutMap,

	// TODO(pat.m): take directly from Input
	click_happened: bool,

	persistent_state: &'ps PersistentState,
}

impl<'ps> Ui<'ps> {
	pub fn new(persistent_state: &'ps PersistentState) -> Ui<'ps> {
		Ui {
			stack: Default::default(),
			layout_constraints: LayoutConstraintMap::default().into(),

			widget_layouts: LayoutMap::new(),

			click_happened: false,

			persistent_state,
		}
	}

	pub fn layout(&mut self, available_bounds: Aabb2) {
		let num_widgets = self.persistent_state.widgets.borrow().len();

		let hierarchy = self.persistent_state.hierarchy.borrow();

		let mut layout_constraints = self.layout_constraints.borrow_mut();
		layout_constraints.reserve(num_widgets);

		// bottom up request size hints/policies and content sizes if appropriate
		hierarchy.visit_leaves_first(None, |widget_id| {
			let mut constraints = layout_constraints.get(&widget_id).cloned().unwrap_or_default();
			let children = hierarchy.children(widget_id);

			self.persistent_state.widgets.borrow_mut().get_mut(&widget_id).unwrap().constrain(ConstraintContext {
				constraints: &mut constraints,
				children,
				constraint_map: &mut layout_constraints,
			});

			layout_constraints.insert(widget_id, constraints);
		});

		self.widget_layouts.clear();
		self.widget_layouts.reserve(num_widgets);

		for &widget_id in hierarchy.root_node.children.iter() {
			layout_children_linear(available_bounds, Axis::Horizontal, Align::Start, &[widget_id], &layout_constraints, &mut self.widget_layouts);
		}

		// top down resolve layouts and assign rects
		hierarchy.visit_breadth_first(None, |widget_id, children| {
			// this widget should already be laid out or root
			let content_bounds = self.widget_layouts[&widget_id].content_bounds;
			let constraints = &layout_constraints[&widget_id];
			let main_axis = constraints.layout_axis.get();
			let content_alignment = constraints.content_alignment.get();

			// TODO(pat.m): layout mode?
			layout_children_linear(content_bounds, main_axis, content_alignment, children, &layout_constraints, &mut self.widget_layouts);
		});
	}

	pub fn handle_input(&mut self, input: &Input) {
		self.persistent_state.hovered_widget.set(None);

		if let Some(cursor_pos) = input.cursor_pos_view {
			self.persistent_state.hierarchy.borrow()
				.visit_breadth_first(None, |widget_id, _| {
					let box_bounds = self.widget_layouts[&widget_id].box_bounds;
					if box_bounds.contains_point(cursor_pos) {
						self.persistent_state.hovered_widget.set(Some(widget_id));
					}
				});
		}
	}

	pub fn draw(&mut self, painter: &mut Painter) {
		// draw from root to leaves
		self.persistent_state.hierarchy.borrow()
			.visit_breadth_first(None, |widget_id, _| {
				let layout = &self.widget_layouts[&widget_id];
				self.persistent_state.widgets.borrow_mut().get_mut(&widget_id).unwrap().draw(painter, layout);
			});
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

	pub fn add_widget_to<T>(&self, mut widget: T, parent_id: impl Into<Option<WidgetId>>) -> WidgetRef<'_, T>
		where T: Widget
	{
		let mut hierarchy = self.persistent_state.hierarchy.borrow_mut();
		let mut widgets = self.persistent_state.widgets.borrow_mut();

		let parent_id = parent_id.into();
		let type_id = TypeId::of::<T>();

		// TODO(pat.m): make id fragment overrideable
		let NodeUpdateResult {
			widget_id,
			status,
		} = hierarchy.add_or_update(WidgetIdFragment::TypedOrdered(type_id), parent_id);

		match status {
			NodeUpdateStatus::Added => {
				let mut widget_state = Box::new(widget);
				widget_state.as_mut().lifecycle(WidgetLifecycleEvent::Created);
				widgets.insert(widget_id, widget_state);
			}

			NodeUpdateStatus::Update => {
				let widget_state = widgets.get_mut(&widget_id).unwrap();
				// TODO(pat.m): this sucks - update should be called with persistent state on new widget
				widget_state.as_mut().lifecycle(WidgetLifecycleEvent::Updated);
				// widget_state.as_mut().update(&mut widget);
				widgets.insert(widget_id, Box::new(widget));
			}
		}

		WidgetRef { widget_id, ui: self, phantom: PhantomData }
	}

	pub fn mutate_widget_constraints(&self, widget_id: impl Into<WidgetId>, mutate: impl FnOnce(&mut LayoutConstraints)) {
		let mut lcs = self.layout_constraints.borrow_mut();
		mutate(lcs.entry(widget_id.into()).or_default());
	}

	pub fn push_layout(&self, widget_id: impl Into<WidgetId>) {
		self.stack.borrow_mut().push(widget_id.into());
	}

	pub fn pop_layout(&self) {
		self.stack.borrow_mut().pop().expect("Parent stack empty!");
	}
}


impl Ui<'_> {
	pub fn dummy(&self) -> WidgetRef<'_, ()> {
		self.add_widget(())
	}

	pub fn spring(&self, axis: Axis) -> WidgetRef<'_, Spring> {
		// TODO(pat.m): can I derive Axis from context?
		self.add_widget(Spring(axis))
	}
}

