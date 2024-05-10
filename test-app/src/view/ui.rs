use crate::prelude::*;

pub mod layout;

pub use layout::*;


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
	layout_constraints: RefCell<LayoutConstraintMap>,

	widget_boxes: SecondaryMap<WidgetId, WidgetBox>,
}

impl Ui {
	pub fn new() -> Ui {
		let mut widgets = WidgetContainer::default();
		let mut hierarchy = Hierarchy::default();
		let mut stack = Vec::new();

		let root_widget = BoxLayout{};

		let root_id = widgets.insert(Box::new(root_widget));
		stack.push(root_id);
		hierarchy.add(root_id, None);

		Ui {
			widgets: widgets.into(),
			hierarchy: hierarchy.into(),
			stack: stack.into(),
			layout_constraints: LayoutConstraintMap::default().into(),

			widget_boxes: SecondaryMap::new(),
		}
	}

	pub fn parent_id(&self) -> WidgetId {
		self.stack.borrow().last().copied()
			.expect("Stack empty")
	}

	pub fn add_widget(&self, widget: impl Widget + 'static) -> WidgetId {
		self.add_widget_to(widget, self.parent_id())
	}

	pub fn add_widget_to(&self, widget: impl Widget + 'static, parent_id: impl Into<Option<WidgetId>>) -> WidgetId {
		let widget_id = self.widgets.borrow_mut().insert(Box::new(widget));
		self.hierarchy.borrow_mut().add(widget_id, parent_id.into());
		widget_id
	}

	pub fn mutate_widget_constraints(&self, widget_id: WidgetId, mutate: impl FnOnce(&mut LayoutConstraints)) {
		let mut lcs = self.layout_constraints.borrow_mut();
		mutate(lcs.entry(widget_id).unwrap().or_default());
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
		// draw from root to leaves
		self.hierarchy.borrow()
			.visit_breadth_first(None, |widget_id, _| {
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

	pub fn children(&self, widget_id: impl Into<Option<WidgetId>>) -> &[WidgetId] {
		match widget_id.into() {
			Some(widget_id) => self.info[widget_id].children.as_slice(),
			None => self.root_nodes.as_slice(),
		}
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

#[derive(Default, Debug, Copy, Clone)]
pub struct BoxLengths {
	pub top: f32,
	pub bottom: f32,
	pub left: f32,
	pub right: f32,
}

impl BoxLengths {
	pub fn zero() -> BoxLengths {
		BoxLengths::default()
	}

	pub fn new(top: f32, bottom: f32, left: f32, right: f32) -> BoxLengths {
		BoxLengths {top, bottom, left, right}
	}

	pub fn uniform(v: f32) -> BoxLengths {
		BoxLengths::new(v, v, v, v)
	}

	pub fn hv(h: f32, v: f32) -> BoxLengths {
		BoxLengths::new(v, v, h, h)
	}
}

impl From<f32> for BoxLengths {
	fn from(o: f32) -> Self {
		BoxLengths::uniform(o)
	}
}

impl From<Vec2> for BoxLengths {
	fn from(o: Vec2) -> Self {
		BoxLengths::hv(o.x, o.y)
	}
}

impl From<(f32, f32)> for BoxLengths {
	fn from((x, y): (f32, f32)) -> Self {
		BoxLengths::hv(x, y)
	}
}


#[derive(Debug, Clone)]
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



pub struct ConstraintContext<'a> {
	pub constraints: &'a mut LayoutConstraints,
	pub children: &'a [WidgetId],
	pub constraint_map: &'a LayoutConstraintMap,
}


pub trait Widget : std::fmt::Debug {
	fn calculate_constraints(&self, _: ConstraintContext<'_>) {}

	fn draw(&self, painter: &mut Painter, widget_box: &WidgetBox) {
		let widget_color = [0.5; 3];
		painter.rect_outline(widget_box.box_bounds, widget_color);
		painter.line(widget_box.box_bounds.min, widget_box.box_bounds.max, widget_color);
		painter.line(widget_box.box_bounds.min_max_corner(), widget_box.box_bounds.max_min_corner(), widget_color);

		painter.rect_outline(widget_box.margin_bounds, [0.1, 0.5, 1.0, 0.5]);
		painter.rect_outline(widget_box.content_bounds, [0.5, 1.0, 0.5, 0.5]);
	}
}

impl Widget for () {
	fn calculate_constraints(&self, ctx: ConstraintContext<'_>) {
		// ctx.constraints.content_width = 50.0;
		// ctx.constraints.content_height = 50.0;
		// ctx.constraints.min_width = 50.0;
		// ctx.constraints.min_height = 20.0;
		ctx.constraints.margin = BoxLengths::from(4.0);
		ctx.constraints.padding = BoxLengths::from(4.0);

		ctx.constraints.set_default(SetConstraints::CONTENT_WIDTH, |c| c.content_width = 50.0);
		ctx.constraints.set_default(SetConstraints::CONTENT_HEIGHT, |c| c.content_height = 50.0);
		ctx.constraints.set_default(SetConstraints::MIN_WIDTH, |c| c.min_width = 20.0);
		ctx.constraints.set_default(SetConstraints::MIN_HEIGHT, |c| c.min_height = 20.0);
	}
}


#[derive(Debug)]
pub struct BoxLayout{}

impl Widget for BoxLayout {
	fn calculate_constraints(&self, ctx: ConstraintContext<'_>) {
		let ConstraintContext { constraints, children, constraint_map } = ctx;

		let padding = 4.0;

		let mut width = 0.0f32;
		let mut height = 0.0f32;

		let mut prev_margin = 0.0f32;

		for &child in children {
			let child_constraints = &constraint_map[child];

			let margin_advance = prev_margin.max(child_constraints.margin.left);
			prev_margin = child_constraints.margin.right;

			width += child_constraints.desired_width() + margin_advance;

			let local_height = child_constraints.desired_height();
			height = height.max(local_height + child_constraints.margin.top + child_constraints.margin.bottom);
		}

		width += prev_margin;

		constraints.content_width = width;
		constraints.content_height = height;
		constraints.padding = BoxLengths::from(padding);
		// constraints.margin = BoxLengths::from(4.0);
	}

	fn draw(&self, painter: &mut Painter, widget_box: &WidgetBox) {
		let widget_color = Color::grey_a(1.0, 0.01);
		painter.rect(widget_box.box_bounds, widget_color);
		painter.rect_outline(widget_box.box_bounds, Color::grey_a(1.0, 0.04));
		// painter.line(widget_box.box_bounds.min, widget_box.box_bounds.max, widget_color);
		// painter.line(widget_box.box_bounds.min_max_corner(), widget_box.box_bounds.max_min_corner(), widget_color);

		// painter.rect_outline(widget_box.content_bounds, [0.1, 0.4, 0.1]);
		// painter.rect_outline(widget_box.margin_bounds, [0.1, 0.1, 0.4]);
	}
}




