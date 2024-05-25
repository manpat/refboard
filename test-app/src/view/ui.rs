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

use std::any::{TypeId, Any};
use std::marker::PhantomData;
// use std::hash::{DefaultHasher, Hasher, Hash};


use cosmic_text as ct;


#[derive(Debug)]
pub struct TextState {
	pub font_system: ct::FontSystem,
	pub swash_cache: ct::SwashCache,

	// TODO(pat.m): text atlas
}


pub struct System {
	persistent_state: PersistentState,
	pub text_state: RefCell<TextState>,
}

impl System {
	pub fn new() -> System {
		let font_system = ct::FontSystem::new();
		let swash_cache = ct::SwashCache::new();

		System {
			persistent_state: PersistentState::default(),

			text_state: TextState {
				font_system,
				swash_cache,
			}.into(),
		}
	}

	// TODO(pat.m): could this be built around the same mechanism as std::thread::scope?
	pub fn run(&mut self, bounds: Aabb2, painter: &mut Painter, input: &Input, build_ui: impl FnOnce(&Ui<'_>)) {
		self.persistent_state.hierarchy.get_mut().new_epoch();

		self.process_input(input);

		let mut ui = Ui::new(&self.persistent_state, input, &self.text_state);

		build_ui(&ui);

		self.garbage_collect();

		ui.layout(bounds);
		ui.handle_input(input);
		ui.draw(painter);
	}

	fn process_input(&mut self, input: &Input) {
		self.persistent_state.hovered_widget.set(None);

		// TODO(pat.m): collect input behaviour from existing widgets

		if let Some(cursor_pos) = input.cursor_pos_view {
			let input_handlers = self.persistent_state.input_handlers.borrow();

			// TODO(pat.m): instead of just storing the last hovered widget, store a 'stack' of hovered widgets
			self.persistent_state.hierarchy.borrow()
				.visit_breadth_first(None, |widget_id, _| {
					if input_handlers[&widget_id].contains_point(cursor_pos) {
						self.persistent_state.hovered_widget.set(Some(widget_id));
					}
				});

			// TODO(pat.m): from the input behaviour of each widget in the hovered widget stack, calculate the target of 
			// any mouse click/keyboard events.
		}
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
	// TODO(pat.m): we don't actually really need this across frames
	widget: Box<dyn Widget>,
	state: StateBox,
}


#[derive(Debug, Default)]
pub struct PersistentState {
	widgets: RefCell<HashMap<WidgetId, WidgetBox>>,
	input_handlers: RefCell<HashMap<WidgetId, Aabb2>>,
	hierarchy: RefCell<Hierarchy>,

	hovered_widget: Cell<Option<WidgetId>>,
}


#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct WidgetId(u64);



#[derive(Debug)]
pub struct Ui<'ps> {
	stack: RefCell<Vec<WidgetId>>,

	widget_constraints: RefCell<LayoutConstraintMap>,
	widget_layouts: LayoutMap,

	persistent_state: &'ps PersistentState,
	pub text_state: &'ps RefCell<TextState>,
	pub input: &'ps Input,
}

impl<'ps> Ui<'ps> {
	fn new(persistent_state: &'ps PersistentState, input: &'ps Input, text_state: &'ps RefCell<TextState>) -> Ui<'ps> {
		Ui {
			stack: Default::default(),
			widget_constraints: LayoutConstraintMap::default().into(),

			widget_layouts: LayoutMap::new(),

			persistent_state,
			text_state,
			input,
		}
	}

	fn layout(&mut self, available_bounds: Aabb2) {
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
			widget_state.widget.constrain(ConstraintContext {
				constraints: &mut constraints,
				children,
				constraint_map: &mut widget_constraints,

				state: &mut widget_state.state,
				text_state,
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
	}

	fn handle_input(&mut self, _input: &Input) {
		// Persist widget bounds
		let mut input_handlers = self.persistent_state.input_handlers.borrow_mut();
		input_handlers.clear();

		self.persistent_state.hierarchy.borrow()
			.visit_breadth_first(None, |widget_id, _| {
				// TODO(pat.m): clipping! also we probably want some per-widget configuration of input behaviour
				let box_bounds = self.widget_layouts[&widget_id].box_bounds;
				input_handlers.insert(widget_id, box_bounds);
			});
	}

	fn draw(&mut self, painter: &mut Painter) {
		let mut widgets = self.persistent_state.widgets.borrow_mut();
		let hierarchy = self.persistent_state.hierarchy.borrow();

		// TODO(pat.m): this is yucky - maybe we should actually calculate clip rects during layout?
		// TODO(pat.m): also we may eventually want widgets to be able to ignore the parent clip rect

		let mut text_state = self.text_state.borrow_mut();
		let text_state = &mut *text_state;

		// draw from root to leaves
		let null_clip_rect = Aabb2::new(Vec2::zero(), Vec2::splat(u16::MAX as f32));
		hierarchy.visit_breadth_first_with_parent_context(null_clip_rect, |widget_id, parent_clip| {
			let layout = &self.widget_layouts[&widget_id];
			let widget_state = widgets.get_mut(&widget_id).unwrap();

			let clip_rect = to_4u16(&parent_clip);

			painter.set_clip_rect(clip_rect);
			widget_state.widget.draw(DrawContext {
				painter,
				layout,
				text_state,

				state: &mut widget_state.state
			});

			// Visualise clip rects
			// painter.set_clip_rect(to_4u16(&null_clip_rect));
			// painter.rect_outline(parent_clip, Color::white());

			clip_rects(&layout.box_bounds, &parent_clip)
		});

		fn to_4u16(bounds: &Aabb2) -> [u16; 4] {
			let min_x = bounds.min.x.round().clamp(0.0, u16::MAX as f32) as i32 as u16;
			let max_x = bounds.max.x.round().clamp(0.0, u16::MAX as f32) as i32 as u16;
			let min_y = bounds.min.y.round().clamp(0.0, u16::MAX as f32) as i32 as u16;
			let max_y = bounds.max.y.round().clamp(0.0, u16::MAX as f32) as i32 as u16;
			[min_x, max_x, min_y, max_y]
		}

		// TODO(pat.m): why is this not on Aabb2
		fn clip_rects(lhs: &Aabb2, rhs: &Aabb2) -> Aabb2 {
			Aabb2 {
				min: Vec2::new(lhs.min.x.max(rhs.min.x), lhs.min.y.max(rhs.min.y)),
				max: Vec2::new(lhs.max.x.min(rhs.max.x), lhs.max.y.min(rhs.max.y)),
			}
		}




		// painter.set_clip_rect(to_4u16(&null_clip_rect));

		// let metrics = ct::Metrics::new(24.0, 32.0);
		// let mut buffer = ct::Buffer::new(&mut text_state.font_system, metrics);
		// {
		// 	let mut buffer = buffer.borrow_with(&mut text_state.font_system);
		// 	buffer.set_size(500.0, 80.0);

		// 	let attrs = ct::Attrs::new();

		// 	buffer.set_text("Hello, Rust! 🦀 🐄🦐🅱 I'm emoting العربية ", attrs, ct::Shaping::Advanced);
		// 	buffer.shape_until_scroll(true);
		// }

		// for run in buffer.layout_runs() {
		// 	for glyph in run.glyphs.iter() {
		// 		let physical_glyph = glyph.physical((0., 0.), 1.0);

		// 		let glyph_color = match glyph.color_opt {
		// 			Some(some) => some,
		// 			None => ct::Color::rgb(0xFF, 0x55, 0xFF),
		// 		};

		// 		let Some(commands) = text_state.swash_cache.get_outline_commands(&mut text_state.font_system, physical_glyph.cache_key)
		// 		else { continue };

		// 		let to_point = |[x, y]: [f32; 2]| {
		// 			lyon::math::Point::new(x + physical_glyph.x as f32 + 100.0, -y + physical_glyph.y as f32 + 300.0)
		// 		};

		// 		let mut builder = lyon::path::Path::builder().with_svg();

		// 		for command in commands {
		// 			match *command {
		// 				ct::Command::MoveTo(p) => { builder.move_to(to_point(p.into())); }
		// 				ct::Command::LineTo(p) => { builder.line_to(to_point(p.into())); }
		// 				ct::Command::CurveTo(c1, c2, to) => { builder.cubic_bezier_to(to_point(c1.into()), to_point(c2.into()), to_point(to.into())); }
		// 				ct::Command::QuadTo(c, to) => { builder.quadratic_bezier_to(to_point(c.into()), to_point(to.into())); }
		// 				ct::Command::Close => { builder.close(); }
		// 			}
		// 		}

		// 		painter.fill_path(&builder.build(), Color::from(glyph_color.as_rgba()).to_linear());
		// 	}
		// }


		// // let text_color = ct::Color::rgb(0xFF, 0xFF, 0xFF);
		// // buffer.draw(&mut text_state.swash_cache, text_color, |x, y, w, h, color| {
		// // 	let pos = Vec2::new(x as f32 + 100.0, y as f32 + 300.0);
		// // 	let size = Vec2::new(w as f32 + 1.0, h as f32 + 1.0);
		// // 	let rect = Aabb2::new(pos, pos + size);

		// // 	painter.rect(rect, Color::from(color.as_rgba()).to_linear());
		// // });
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
				};

				widget_box.widget.lifecycle(LifecycleContext {
					event: WidgetLifecycleEvent::Created,
					state: &mut widget_box.state,
					text_state,
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
}