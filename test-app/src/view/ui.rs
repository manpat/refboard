use crate::prelude::*;



pub struct System {
}

impl System {
	pub fn new() -> System {
		System{}
	}

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
	widgets: RefCell<WidgetContainer>,
	hierarchy: RefCell<Hierarchy>,
	stack: RefCell<Vec<WidgetId>>,

	widget_boxes: SecondaryMap<WidgetId, WidgetBox>,
}

impl Ui {
	pub fn new() -> Ui {
		let mut widgets = WidgetContainer::default();
		let mut hierarchy = Hierarchy::default();
		let mut stack = Vec::new();

		let root_widget = ();

		let root_id = widgets.insert(Box::new(root_widget));
		stack.push(root_id);
		hierarchy.add(root_id, None);

		Ui {
			widgets: widgets.into(),
			hierarchy: hierarchy.into(),
			stack: stack.into(),

			widget_boxes: SecondaryMap::new(),
		}
	}

	pub fn parent_id(&self) -> WidgetId {
		self.stack.borrow().last().copied()
			.expect("Stack empty")
	}

	pub fn add_widget(&self, widget: impl Widget + 'static) -> WidgetId {
		let widget_id = self.widgets.borrow_mut().insert(Box::new(widget));
		self.hierarchy.borrow_mut().add(widget_id, self.parent_id());
		widget_id
	}

	pub fn layout(&mut self, available_bounds: Aabb2) {
		println!("--------------- layout -----------------");
		let num_widgets = self.widgets.borrow().len();
		let mut layout_constraints = SecondaryMap::with_capacity(num_widgets);

		let hierarchy = self.hierarchy.borrow();

		// bottom up request size hints/policies and content sizes if appropriate
		hierarchy.visit_leaves_first(None, |widget_id| {
			let constraints = layout_constraints.entry(widget_id).unwrap().or_default();
			self.widgets.borrow_mut()[widget_id].layout_constraints(constraints);
		});

		self.widget_boxes.clear();
		self.widget_boxes.set_capacity(num_widgets);

		for &widget_id in hierarchy.root_nodes.iter() {
			layout(available_bounds, &[widget_id], &layout_constraints, &mut self.widget_boxes);
		}

		// top down resolve layouts and assign rects
		hierarchy.visit_breadth_first(None, |widget_id, children| {
			// this widget should already be laid out or root
			let content_bounds = self.widget_boxes[widget_id].content_bounds;

			// TODO(pat.m): layout mode?
			layout(content_bounds, children, &layout_constraints, &mut self.widget_boxes);
		});
	}

	pub fn draw(&mut self, painter: &mut Painter) {
		println!("--------------- draw -----------------");
		// draw from root to leaves
		self.hierarchy.borrow()
			.visit_breadth_first(None, |widget_id, children| {
				self.widgets.borrow_mut()[widget_id].draw(painter, &self.widget_boxes[widget_id]);
			});
	}
}


impl Ui {
	pub fn dummy(&self) {
		self.add_widget(());
	}
}


#[derive(Default, Debug)]
pub struct Hierarchy {
	pub info: SecondaryMap<WidgetId, HierarchyNode>,
	pub root_nodes: Vec<WidgetId>,
}

#[derive(Debug)]
pub struct HierarchyNode {
	pub parent: Option<WidgetId>,
	pub children: Vec<WidgetId>,
}

impl Hierarchy {
	pub fn add(&mut self, widget_id: WidgetId, parent: impl Into<Option<WidgetId>>) {
		let parent = parent.into();

		if let Some(parent) = parent {
			self.info[parent].children.push(widget_id);
		} else {
			self.root_nodes.push(widget_id);
		}

		let prev_value = self.info.insert(widget_id, HierarchyNode{parent, children: Vec::new()});
		assert!(prev_value.is_none(), "Widget already added to hierarchy!");
	}

	pub fn visit_breadth_first<F>(&self, start: impl Into<Option<WidgetId>>, mut visit: F)
		where F: FnMut(WidgetId, &[WidgetId])
	{
		let start = start.into();
		let children = match start {
			Some(widget_id) => self.info[widget_id].children.as_slice(),
			None => self.root_nodes.as_slice(),
		};

		// TODO(pat.m): reuse intermediate visit structures
		let mut visit_queue = VecDeque::new();

		visit_queue.extend(children.into_iter());

		while let Some(parent) = visit_queue.pop_front() {
			let children = self.info[parent].children.as_slice();
			visit_queue.extend(children.iter().copied());

			visit(parent, children);
		}
	}

	/// Postorder traversal
	pub fn visit_leaves_first<F>(&self, start: impl Into<Option<WidgetId>>, mut visit: F)
		where F: FnMut(WidgetId)
	{
		// TODO(pat.m): reuse intermediate visit structures
		let mut visit_stack = Vec::new();

		let start = start.into();
		match start {
			Some(widget_id) => {
				visit_stack.push((widget_id, false));
			}

			None => {
				visit_stack.extend(self.root_nodes.iter().rev().map(|&id| (id, false)));
			}
		};

		while let Some((parent, children_visited)) = visit_stack.pop() {
			if children_visited {
				visit(parent);
				continue
			}

			let children = self.info[parent].children.as_slice();
			if children.is_empty() {
				visit(parent);
				continue
			}

			visit_stack.push((parent, true));
			visit_stack.extend(children.iter().rev().map(|&id| (id, false)));
		}
	}
}


#[derive(Default, Debug)]
pub struct LayoutConstraints {
	// min max size
	// content size
	// size policies
	// alignment
	// margin
	// padding

	// children layout policy
}

#[derive(Debug)]
pub struct WidgetBox {
	pub box_bounds: Aabb2,
	pub margin_bounds: Aabb2,
	pub content_bounds: Aabb2,
}

impl Default for WidgetBox {
	fn default() -> Self {
		WidgetBox {
			box_bounds: Aabb2::zero(),
			margin_bounds: Aabb2::zero(),
			content_bounds: Aabb2::zero(),
		}
	}
}



pub trait Widget : std::fmt::Debug {
	fn layout_constraints(&self, _: &mut LayoutConstraints) {}

	fn draw(&self, painter: &mut Painter, widget_box: &WidgetBox) {
		painter.rect_outline(widget_box.box_bounds, [1.0; 3]);
		dbg!(widget_box);
	}
}




impl Widget for () {
}



fn layout(
	available_bounds: Aabb2,
	widgets: &[WidgetId],
	constraints: &SecondaryMap<WidgetId, LayoutConstraints>,
	widget_boxes: &mut SecondaryMap<WidgetId, WidgetBox>)
{
	for &widget_id in widgets {
		widget_boxes.insert(widget_id, WidgetBox {
			// TODO(pat.m): take margin into account
			box_bounds: available_bounds,

			// TODO(pat.m): take padding into account
			content_bounds: available_bounds.shrink(Vec2::splat(4.0)),
			margin_bounds: available_bounds,
		});
	}
}