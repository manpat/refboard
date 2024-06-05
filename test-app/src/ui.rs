use crate::prelude::*;

pub mod widget;
pub mod widget_ref;
pub mod widgets;
pub mod style;
pub mod state_box;
pub mod system;
pub mod text;
pub mod layout;
pub mod hierarchy;
pub mod input;
pub mod viewport;
pub mod widget_config;

pub use widget::*;
pub use widgets::*;
pub use widget_ref::*;
pub use widget_config::*;
pub use style::*;
pub use state_box::*;
pub use system::*;
pub use text::*;
pub use layout::*;
pub use hierarchy::*;
pub use viewport::*;
pub use input::*;

use std::any::{TypeId, Any};
use std::marker::PhantomData;




#[derive(Debug)]
struct WidgetBox {
	widget: Box<dyn Widget>,
	state: StateBox,

	// TODO(pat.m): this shouldn't really be retained, just not sure where to put it
	config: WidgetConfiguration,
}


pub struct PersistentState {
	widgets: RefCell<HashMap<WidgetId, WidgetBox>>,
	hierarchy: RefCell<Hierarchy>,

	style: AppStyle,
}

impl PersistentState {
	pub fn new() -> Self {
		PersistentState {
			widgets: Default::default(),
			hierarchy: Default::default(),

			style: AppStyle::new(),
		}
	}
}


pub struct Ui<'ps> {
	stack: RefCell<Vec<WidgetId>>,
	widget_constraints: &'ps RefCell<LayoutConstraintMap>,

	persistent_state: &'ps PersistentState,
	pub text_state: &'ps RefCell<TextState>,
	pub input: &'ps Input,
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

					state: StateBox::empty(),
					config: WidgetConfiguration::default(),
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

				widget_box.config = WidgetConfiguration::default();

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

