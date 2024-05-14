use crate::prelude::*;

pub mod widget;
pub mod widgets;
pub mod layout;
pub mod hierarchy;

pub use widget::*;
pub use widgets::*;
pub use layout::*;
pub use hierarchy::*;

use std::marker::PhantomData;


pub struct System {}

impl System {
	pub fn new() -> System {
		System{}
	}

	// TODO(pat.m): could this be built around the same mechanism as std::thread::scope?
	pub fn run(&mut self, bounds: Aabb2, painter: &mut Painter, build_ui: impl FnOnce(&Ui)) {
		let mut ui = Ui::new();
		build_ui(&ui);

		ui.layout(bounds);
		ui.draw(painter);
	}
}


slotmap::new_key_type! {
	pub struct WidgetId;
}

type WidgetContainer = SlotMap<WidgetId, Box<dyn Widget>>;


#[derive(Debug)]
pub struct Ui {
	// TODO(pat.m): most of this shouldn't be slotmaps - we never remove widgets within a frame so their index is stable
	widgets: RefCell<WidgetContainer>,
	hierarchy: RefCell<Hierarchy>,
	stack: RefCell<Vec<WidgetId>>,
	layout_constraints: RefCell<LayoutConstraintMap>,

	widget_layouts: SecondaryMap<WidgetId, Layout>,
}

impl Ui {
	pub fn new() -> Ui {
		let mut widgets = WidgetContainer::default();
		let mut hierarchy = Hierarchy::default();
		let mut stack = Vec::new();

		let root_widget = BoxLayout{ axis: Axis::Horizontal };

		let root_id = widgets.insert(Box::new(root_widget));
		stack.push(root_id);
		hierarchy.add(root_id, None);

		Ui {
			widgets: widgets.into(),
			hierarchy: hierarchy.into(),
			stack: stack.into(),
			layout_constraints: LayoutConstraintMap::default().into(),

			widget_layouts: SecondaryMap::new(),
		}
	}

	pub fn layout(&mut self, available_bounds: Aabb2) {
		let num_widgets = self.widgets.borrow().len();

		let hierarchy = self.hierarchy.borrow();

		let mut layout_constraints = self.layout_constraints.borrow_mut();
		layout_constraints.set_capacity(num_widgets);

		// bottom up request size hints/policies and content sizes if appropriate
		hierarchy.visit_leaves_first(None, |widget_id| {
			let mut constraints = layout_constraints.get(widget_id).cloned().unwrap_or_default();
			let children = hierarchy.children(widget_id);

			self.widgets.borrow_mut()[widget_id].calculate_constraints(ConstraintContext {
				constraints: &mut constraints,
				children,
				constraint_map: &mut layout_constraints,
			});

			layout_constraints.insert(widget_id, constraints);
		});

		self.widget_layouts.clear();
		self.widget_layouts.set_capacity(num_widgets);

		for &widget_id in hierarchy.root_nodes.iter() {
			layout_children_linear(available_bounds, Axis::Horizontal, Align::Start, &[widget_id], &layout_constraints, &mut self.widget_layouts);
		}

		// top down resolve layouts and assign rects
		hierarchy.visit_breadth_first(None, |widget_id, children| {
			// this widget should already be laid out or root
			let content_bounds = self.widget_layouts[widget_id].content_bounds;
			let constraints = &layout_constraints[widget_id];
			let main_axis = constraints.layout_axis.get();
			let content_alignment = constraints.content_alignment.get();

			// TODO(pat.m): layout mode?
			layout_children_linear(content_bounds, main_axis, content_alignment, children, &layout_constraints, &mut self.widget_layouts);
		});
	}

	pub fn draw(&mut self, painter: &mut Painter) {
		// draw from root to leaves
		self.hierarchy.borrow()
			.visit_breadth_first(None, |widget_id, _| {
				self.widgets.borrow_mut()[widget_id].draw(painter, &self.widget_layouts[widget_id]);
			});
	}
}


impl Ui {
	pub fn parent_id(&self) -> WidgetId {
		self.stack.borrow().last().copied()
			.expect("Stack empty")
	}

	pub fn add_widget<T>(&self, widget: T) -> WidgetRef<'_, T>
		where T: Widget + 'static
	{
		self.add_widget_to(widget, self.parent_id())
	}

	pub fn add_widget_to<T>(&self, widget: T, parent_id: impl Into<Option<WidgetId>>) -> WidgetRef<'_, T>
		where T: Widget + 'static
	{
		let widget_id = self.widgets.borrow_mut().insert(Box::new(widget));
		self.hierarchy.borrow_mut().add(widget_id, parent_id.into());

		// TODO(pat.m): calc some kind of key that can be used across frames for input purposes

		WidgetRef { widget_id, ui: self, phantom: PhantomData }
	}

	pub fn mutate_widget_constraints(&self, widget_id: impl Into<WidgetId>, mutate: impl FnOnce(&mut LayoutConstraints)) {
		let mut lcs = self.layout_constraints.borrow_mut();
		mutate(lcs.entry(widget_id.into()).unwrap().or_default());
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