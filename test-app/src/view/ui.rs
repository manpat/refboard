use crate::prelude::*;

pub mod layout;
pub mod hierarchy;

pub use layout::*;
pub use hierarchy::*;


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
	pub fn dummy(&self) {
		self.add_widget(());
	}
}




pub struct ConstraintContext<'a> {
	pub constraints: &'a mut LayoutConstraints,
	pub children: &'a [WidgetId],
	pub constraint_map: &'a LayoutConstraintMap,
}


pub trait Widget : std::fmt::Debug {
	fn calculate_constraints(&self, _: ConstraintContext<'_>) {}

	fn draw(&self, painter: &mut Painter, layout: &Layout) {
		let widget_color = [0.5; 3];
		painter.rounded_rect_outline(layout.box_bounds, 8.0, widget_color);
		painter.line(layout.box_bounds.min, layout.box_bounds.max, widget_color);
		painter.line(layout.box_bounds.min_max_corner(), layout.box_bounds.max_min_corner(), widget_color);

		painter.rounded_rect_outline(layout.margin_bounds, 8.0, [0.1, 0.5, 1.0, 0.5]);
		painter.rounded_rect_outline(layout.content_bounds, 8.0, [0.5, 1.0, 0.5, 0.5]);
	}
}

impl Widget for () {
	fn calculate_constraints(&self, ctx: ConstraintContext<'_>) {
		ctx.constraints.margin.set_default(16.0);
		ctx.constraints.padding.set_default(16.0);

		// ctx.constraints.preferred_width.set_default(50.0);
		// ctx.constraints.preferred_height.set_default(50.0);
		// ctx.constraints.min_width.set_default(10.0);
		// ctx.constraints.min_height.set_default(10.0);

		// ctx.constraints.max_width.set_default(50.0);
		// ctx.constraints.max_height.set_default(50.0);

		ctx.constraints.horizontal_size_policy.set_default(SizingBehaviour::FLEXIBLE);
	}
}


#[derive(Debug)]
pub struct BoxLayout {
	pub axis: Axis,
}

impl Widget for BoxLayout {
	fn calculate_constraints(&self, ctx: ConstraintContext<'_>) {
		let ConstraintContext { constraints, children, constraint_map } = ctx;

		constraints.layout_axis.set_default(self.axis);

		let main_axis = constraints.layout_axis.get();
		let cross_axis = main_axis.opposite();

		let mut total_main_min_length = 0.0f32;
		let mut total_cross_min_length = 0.0f32;

		let mut total_main_preferred_length = 0.0f32;
		let mut total_cross_preferred_length = 0.0f32;

		for &child in children {
			let child_constraints = &constraint_map[child];

			let margin_main = child_constraints.margin.axis_sum(main_axis);
			let margin_cross = child_constraints.margin.axis_sum(cross_axis);

			let min_length = child_constraints.min_length(main_axis);
			let preferred_length = child_constraints.preferred_length(main_axis);

			total_main_min_length += min_length + margin_main;
			total_main_preferred_length += preferred_length + margin_main;

			total_cross_min_length = total_cross_min_length.max(child_constraints.min_length(cross_axis) + margin_cross);
			total_cross_preferred_length = total_cross_preferred_length.max(child_constraints.preferred_length(cross_axis) + margin_cross);
		}

		constraints.padding.set_default(8.0);

		let padding_main = constraints.padding.axis_sum(main_axis);
		let padding_cross = constraints.padding.axis_sum(cross_axis);

		total_main_min_length += padding_main;
		total_cross_min_length += padding_cross;

		total_main_preferred_length += padding_main;
		total_cross_preferred_length += padding_cross;

		constraints.min_length_mut(main_axis).set_default(total_main_min_length);
		constraints.min_length_mut(cross_axis).set_default(total_cross_min_length);

		constraints.preferred_length_mut(main_axis).set_default(total_main_preferred_length);
		constraints.preferred_length_mut(cross_axis).set_default(total_cross_preferred_length);

		constraints.size_policy_mut(main_axis).set_default(SizingBehaviour::FLEXIBLE);
		constraints.size_policy_mut(cross_axis).set_default(SizingBehaviour::FLEXIBLE);
	}

	fn draw(&self, painter: &mut Painter, layout: &Layout) {
		let widget_color = Color::grey_a(1.0, 0.01);
		painter.rounded_rect(layout.box_bounds, 8.0, widget_color);
		painter.rounded_rect_outline(layout.box_bounds, 8.0, Color::grey_a(1.0, 0.04));
		// painter.line(layout.box_bounds.min, layout.box_bounds.max, widget_color);
		// painter.line(layout.box_bounds.min_max_corner(), layout.box_bounds.max_min_corner(), widget_color);

		painter.rounded_rect_outline(layout.content_bounds, 8.0, [0.1, 0.4, 0.1]);
		painter.rounded_rect_outline(layout.margin_bounds, 8.0, [0.1, 0.1, 0.4]);
	}
}




