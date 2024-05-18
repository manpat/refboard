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
use std::hash::{DefaultHasher, Hasher, Hash};
// use std::cell::Cell;


pub struct System {
	hovered_widget: Option<WidgetId>,
}

impl System {
	pub fn new() -> System {
		System {
			hovered_widget: None,
		}
	}

	// TODO(pat.m): could this be built around the same mechanism as std::thread::scope?
	pub fn run(&mut self, bounds: Aabb2, painter: &mut Painter, input: &Input, build_ui: impl FnOnce(&Ui)) {
		let mut ui = Ui::new();
		ui.hovered_widget = self.hovered_widget;
		ui.click_happened = input.click_received;

		// TODO(pat.m): handle input first, using cached input handlers from previous frame

		build_ui(&ui);

		ui.layout(bounds);
		ui.handle_input(input);
		ui.draw(painter);

		self.hovered_widget = ui.hovered_widget;
	}
}



type WidgetContainer = HashMap<WidgetId, Box<dyn Widget>>;


#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct WidgetId(u64);



#[derive(Debug)]
pub struct Ui {
	// TODO(pat.m): most of this shouldn't be slotmaps - we never remove widgets within a frame so their index is stable
	widgets: RefCell<WidgetContainer>,
	hierarchy: RefCell<Hierarchy>,
	stack: RefCell<Vec<WidgetId>>,
	layout_constraints: RefCell<LayoutConstraintMap>,

	widget_layouts: LayoutMap,

	// TODO(pat.m): Set in Persistent state, not here
	hovered_widget: Option<WidgetId>,

	// TODO(pat.m): take directly from Input
	click_happened: bool,
}

impl Ui {
	pub fn new() -> Ui {
		let widgets = WidgetContainer::default();
		let hierarchy = Hierarchy::default();
		let stack = Vec::new();

		Ui {
			widgets: widgets.into(),
			hierarchy: hierarchy.into(),
			stack: stack.into(),
			layout_constraints: LayoutConstraintMap::default().into(),

			widget_layouts: LayoutMap::new(),

			hovered_widget: None,
			click_happened: false,
		}
	}

	pub fn layout(&mut self, available_bounds: Aabb2) {
		let num_widgets = self.widgets.borrow().len();

		let hierarchy = self.hierarchy.borrow();

		let mut layout_constraints = self.layout_constraints.borrow_mut();
		layout_constraints.reserve(num_widgets);

		// bottom up request size hints/policies and content sizes if appropriate
		hierarchy.visit_leaves_first(None, |widget_id| {
			let mut constraints = layout_constraints.get(&widget_id).cloned().unwrap_or_default();
			let children = hierarchy.children(widget_id);

			self.widgets.borrow_mut().get_mut(&widget_id).unwrap().constrain(ConstraintContext {
				constraints: &mut constraints,
				children,
				constraint_map: &mut layout_constraints,
			});

			layout_constraints.insert(widget_id, constraints);
		});

		self.widget_layouts.clear();
		self.widget_layouts.reserve(num_widgets);

		for &widget_id in hierarchy.root_nodes.iter() {
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
		self.hovered_widget = None;

		if let Some(cursor_pos) = input.cursor_pos_view {
			self.hierarchy.borrow()
				.visit_breadth_first(None, |widget_id, _| {
					let box_bounds = self.widget_layouts[&widget_id].box_bounds;
					if box_bounds.contains_point(cursor_pos) {
						self.hovered_widget = Some(widget_id);
					}
				});
		}
	}

	pub fn draw(&mut self, painter: &mut Painter) {
		// draw from root to leaves
		self.hierarchy.borrow()
			.visit_breadth_first(None, |widget_id, _| {
				let layout = &self.widget_layouts[&widget_id];
				self.widgets.borrow_mut().get_mut(&widget_id).unwrap().draw(painter, layout);
			});
	}
}


impl Ui {
	pub fn parent_id(&self) -> Option<WidgetId> {
		self.stack.borrow().last().copied()
	}

	pub fn add_widget<T>(&self, widget: T) -> WidgetRef<'_, T>
		where T: Widget + 'static
	{
		self.add_widget_to(widget, self.parent_id())
	}

	pub fn add_widget_to<T>(&self, widget: T, parent_id: impl Into<Option<WidgetId>>) -> WidgetRef<'_, T>
		where T: Widget + 'static
	{
		let mut hierarchy = self.hierarchy.borrow_mut();

		let parent_id = parent_id.into();
		let type_id = TypeId::of::<T>();
		let widget_number = hierarchy.children(parent_id).len();

		let mut hasher = DefaultHasher::new();
		parent_id.hash(&mut hasher);
		type_id.hash(&mut hasher);
		hasher.write_usize(widget_number);
		let widget_id = WidgetId(hasher.finish());

		self.widgets.borrow_mut().insert(widget_id, Box::new(widget));
		hierarchy.add(widget_id, parent_id);

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


impl Ui {
	pub fn dummy(&self) -> WidgetRef<'_, ()> {
		self.add_widget(())
	}

	pub fn spring(&self, axis: Axis) -> WidgetRef<'_, Spring> {
		// TODO(pat.m): can I derive Axis from context?
		self.add_widget(Spring(axis))
	}
}

