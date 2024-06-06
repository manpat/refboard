use crate::prelude::*;
use super::*;

pub struct System {
	pub viewport: Viewport,
	pub input: Input,
	pub text_atlas: RefCell<TextAtlas>,
	pub min_size: Vec2i,

	persistent_state: PersistentState,
	should_redraw: bool,

	widget_constraints: RefCell<LayoutConstraintMap>,
	widget_layouts: LayoutMap,
}

impl System {
	pub fn new() -> System {
		System {
			viewport: Viewport::default(),
			input: Input::default(),
			text_atlas: TextAtlas::new().into(),
			min_size: Vec2i::zero(),

			persistent_state: PersistentState::new(),
			should_redraw: true,

			widget_constraints: Default::default(),
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

		self.input.process_events(&self.viewport, self.persistent_state.hierarchy.get_mut());

		self.widget_constraints.get_mut().clear();

		build_ui(&Ui {
			stack: Default::default(),
			widget_constraints: &self.widget_constraints,

			persistent_state: &self.persistent_state,
			input: &self.input,
			text_atlas: &self.text_atlas,
		});

		self.garbage_collect();

		self.layout_widgets();
		self.draw_widgets(painter);

		self.min_size = self.calc_min_size();

		self.persist_input_bounds();
	}

	fn persist_input_bounds(&mut self) {
		// Persist widget bounds
		let input_handlers = &mut self.input.registered_widgets;
		let widgets = self.persistent_state.widgets.get_mut();

		input_handlers.clear();

		self.persistent_state.hierarchy.borrow()
			.visit_breadth_first(None, |widget_id, _| {
				let widget_state = widgets.get_mut(&widget_id).unwrap();
				if widget_state.config.input.contains(InputBehaviour::TRANSPARENT) {
					return
				}

				let box_bounds = self.widget_layouts[&widget_id].box_bounds;

				input_handlers.insert(widget_id, RegisteredWidget {
					bounds: box_bounds,
					behaviour: widget_state.config.input,
				});
			});
	}

	fn garbage_collect(&mut self) {
		let widgets = self.persistent_state.widgets.get_mut();
		let text_atlas = self.text_atlas.get_mut();

		self.persistent_state.hierarchy.get_mut()
			.collect_stale_nodes(|widget_id| {
				if let Some(mut widget_state) = widgets.remove(&widget_id) {
					widget_state.widget.lifecycle(LifecycleContext {
						event: WidgetLifecycleEvent::Destroyed,
						state: &mut widget_state.state,
						text_atlas,
						input: &self.input,
						widget_id,
					});
				}
			});
	}

	fn calc_min_size(&self) -> Vec2i {
		let widget_constraints = self.widget_constraints.borrow();
		let hierarchy = self.persistent_state.hierarchy.borrow();

		let mut min_width = 10.0;
		let mut min_height = 10.0;

		for widget_id in hierarchy.children(None) {
			let constraints = &widget_constraints[widget_id];
			min_width = constraints.min_width.get().max(min_width);
			min_height = constraints.min_height.get().max(min_height);
		}

		Vec2i::new(min_width as i32, min_height as i32)
	}


	fn layout_widgets(&mut self) {
		let available_bounds = self.viewport.view_bounds();

		let hierarchy = self.persistent_state.hierarchy.borrow();
		let widgets = self.persistent_state.widgets.get_mut();
		let widget_constraints = self.widget_constraints.get_mut();
		let text_atlas = self.text_atlas.get_mut();

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
				constraint_map: widget_constraints,

				input: &mut widget_state.config.input,
				style: &mut widget_state.config.style,

				state: &mut widget_state.state,
				text_atlas,
				widget_id,
			});

			widget_constraints.insert(widget_id, constraints);
		});

		self.widget_layouts.clear();
		self.widget_layouts.reserve(num_widgets);

		for &widget_id in hierarchy.root_node.children.iter() {
			layout_children_linear(available_bounds, Axis::Horizontal, Align::Start, &[widget_id], &widget_constraints, &mut self.widget_layouts);
		}

		// top down resolve layouts and assign rects
		hierarchy.visit_breadth_first(None, |widget_id, children| {
			// this widget should already be laid out or root
			let content_bounds = self.widget_layouts[&widget_id].content_bounds;
			let constraints = &widget_constraints[&widget_id];
			let main_axis = constraints.layout_axis.get();
			let content_alignment = constraints.content_alignment.get();

			// TODO(pat.m): layout mode?
			layout_children_linear(content_bounds, main_axis, content_alignment, children, &widget_constraints, &mut self.widget_layouts);
		});

		// Calculate clip rects
		hierarchy.visit_breadth_first_with_parent_context(None, |widget_id, parent_clip| {
			let layout = self.widget_layouts.get_mut(&widget_id).unwrap();
			layout.clip_rect = parent_clip;

			let clip_rect = match parent_clip {
				Some(parent_clip) => clip_rects(&layout.box_bounds, &parent_clip),
				None => layout.box_bounds,
			};

			Some(clip_rect)
		});
	}


	fn draw_widgets(&mut self, painter: &mut Painter) {
		let widgets = self.persistent_state.widgets.get_mut();
		let hierarchy = self.persistent_state.hierarchy.borrow();

		let text_atlas = self.text_atlas.get_mut();
		let app_style = &self.persistent_state.style;

		// draw from root to leaves
		hierarchy.visit_breadth_first(None, |widget_id, _| {
			let layout = &self.widget_layouts[&widget_id];

			// Don't draw if not visible
			if let Some(clip_rect) = layout.clip_rect
				&& !rect_intersects(&clip_rect, &layout.box_bounds)
			{
				return
			}

			let widget_state = widgets.get_mut(&widget_id).unwrap();
			let style = &widget_state.config.style;

			painter.set_clip_rect(layout.clip_rect);

			draw_default_box(painter, &layout.box_bounds, app_style, style);

			widget_state.widget.draw(DrawContext {
				painter,
				layout,
				text_atlas,

				style,
				app_style,

				state: &mut widget_state.state,
				input: &self.input,
				widget_id,
			});
		});

		// Visualise clip rects
		// TODO(pat.m): make this a debug setting
		if false {
			painter.set_clip_rect(None);
			painter.set_color(Color::white());

			hierarchy.visit_breadth_first(None, |widget_id, _| {
				if let Some(clip) = self.widget_layouts[&widget_id].clip_rect {
					painter.rect_outline(clip);
				}
			});
		}
	}
}


// TODO(pat.m): why is this not on Aabb2
fn rect_intersects(lhs: &Aabb2, rhs: &Aabb2) -> bool {
	!(lhs.min.x > rhs.max.x
	|| lhs.min.y > rhs.max.y
	|| lhs.max.x < rhs.min.x
	|| lhs.max.y < rhs.min.y)
}

// TODO(pat.m): why is this not on Aabb2
fn clip_rects(lhs: &Aabb2, rhs: &Aabb2) -> Aabb2 {
	Aabb2 {
		min: Vec2::new(lhs.min.x.max(rhs.min.x), lhs.min.y.max(rhs.min.y)),
		max: Vec2::new(lhs.max.x.min(rhs.max.x), lhs.max.y.min(rhs.max.y)),
	}
}


fn draw_default_box(painter: &mut Painter, bounds: &Aabb2, app_style: &AppStyle, widget_style: &WidgetStyle) {
	let rounding = widget_style.rounding(app_style);

	// Fill
	if let Some(color) = &widget_style.fill {
		painter.set_color(color.resolve(app_style));
		painter.rounded_rect(*bounds, rounding);
	}

	// Outline
	if let Some(outline) = &widget_style.outline {
		painter.set_color(outline.color.resolve(app_style));
		painter.set_line_width(outline.width);
		painter.rounded_rect_outline(*bounds, rounding);
	}
}