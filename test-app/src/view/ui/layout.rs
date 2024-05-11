use crate::prelude::*;
use super::{WidgetId};


pub type LayoutConstraintMap = SecondaryMap<WidgetId, LayoutConstraints>;



#[derive(Debug, Clone, Copy, Default)]
pub struct WidgetParameter<T> {
	value: T,
	set: bool,
}

impl<T> WidgetParameter<T> {
	pub fn new(initial_value: T) -> Self {
		WidgetParameter {
			value: initial_value,
			set: false,
		}
	}

	pub fn set(&mut self, value: T) {
		self.value = value;
		self.set = true;
	}

	pub fn set_default(&mut self, value: T) {
		if !self.set {
			self.value = value;
		}
	}

	pub fn get(&self) -> T
		where T: Copy
	{
		self.value
	}
}



#[derive(Default, Debug, Copy, Clone)]
pub struct BoxLengths {
	pub left: WidgetParameter<f32>,
	pub right: WidgetParameter<f32>,
	pub top: WidgetParameter<f32>,
	pub bottom: WidgetParameter<f32>,
}

impl BoxLengths {
	pub fn zero() -> BoxLengths {
		BoxLengths::default()
	}

	pub fn new(left: f32, right: f32, top: f32, bottom: f32) -> BoxLengths {
		BoxLengths {
			left: WidgetParameter::new(left),
			right: WidgetParameter::new(right),
			top: WidgetParameter::new(top),
			bottom: WidgetParameter::new(bottom),
		}
	}

	pub fn uniform(v: f32) -> BoxLengths {
		BoxLengths::new(v, v, v, v)
	}

	pub fn from_axes(h: f32, v: f32) -> BoxLengths {
		BoxLengths::new(h, h, v, v)
	}

	pub fn set(&mut self, other: impl Into<BoxLengths>) {
		let BoxLengths{left, right, top, bottom} = other.into();
		self.left.set(left.get());
		self.right.set(right.get());
		self.top.set(top.get());
		self.bottom.set(bottom.get());
	}

	pub fn set_default(&mut self, other: impl Into<BoxLengths>) {
		let BoxLengths{left, right, top, bottom} = other.into();
		self.left.set_default(left.get());
		self.right.set_default(right.get());
		self.top.set_default(top.get());
		self.bottom.set_default(bottom.get());
	}

	pub fn set_horizontal(&mut self, value: f32) {
		self.left.set(value);
		self.right.set(value);
	}

	pub fn set_vertical(&mut self, value: f32) {
		self.top.set(value);
		self.bottom.set(value);
	}

	pub fn set_uniform(&mut self, value: f32) {
		self.set_horizontal(value);
		self.set_vertical(value);
	}

	pub fn horizontal_sum(&self) -> f32 {
		self.left.get() + self.right.get()
	}

	pub fn vertical_sum(&self) -> f32 {
		self.top.get() + self.bottom.get()
	}
}

impl From<f32> for BoxLengths {
	fn from(o: f32) -> Self {
		BoxLengths::uniform(o)
	}
}

impl From<Vec2> for BoxLengths {
	fn from(o: Vec2) -> Self {
		BoxLengths::from_axes(o.x, o.y)
	}
}

impl From<(f32, f32)> for BoxLengths {
	fn from((x, y): (f32, f32)) -> Self {
		BoxLengths::from_axes(x, y)
	}
}

bitflags! {
	#[derive(Debug, Clone, Copy)]
	pub struct SizingBehaviour : u16 {
		/// Allow > preferred - if there's spare space
		const CAN_GROW = 1 << 0;

		/// Allow < preferred - if not enough space
		const CAN_SHRINK = 1 << 1;

		/// Preferred from content sizes
		// CONTENT,

		/// Min == Max == Preferred
		const FIXED = 0;

		/// Grow and shrink to fill space between [min, max]
		const FLEXIBLE = Self::CAN_GROW.bits() | Self::CAN_SHRINK.bits();
	}
}


#[derive(Debug, Clone)]
pub struct LayoutConstraints {
	pub min_width: WidgetParameter<f32>,
	pub min_height: WidgetParameter<f32>,

	pub max_width: WidgetParameter<f32>,
	pub max_height: WidgetParameter<f32>,

	// TODO(pat.m): should this include or exclude padding?
	pub preferred_width: WidgetParameter<f32>,
	pub preferred_height: WidgetParameter<f32>,

	pub margin: BoxLengths,
	pub padding: BoxLengths,

	pub horizontal_size_policy: WidgetParameter<SizingBehaviour>,
	pub vertical_size_policy: WidgetParameter<SizingBehaviour>,

	// TODO(pat.m): baselines??

	// size policies
	// alignment

	// children layout policy

	// pub set: SetConstraints,
}

impl Default for LayoutConstraints {
	fn default() -> Self {
		LayoutConstraints {
			min_width: WidgetParameter::new(0.0),
			min_height: WidgetParameter::new(0.0),

			max_width: WidgetParameter::new(f32::INFINITY),
			max_height: WidgetParameter::new(f32::INFINITY),

			preferred_width: WidgetParameter::new(0.0),
			preferred_height: WidgetParameter::new(0.0),

			margin: BoxLengths::default(),
			padding: BoxLengths::default(),

			horizontal_size_policy: WidgetParameter::new(SizingBehaviour::FLEXIBLE),
			vertical_size_policy: WidgetParameter::new(SizingBehaviour::FLEXIBLE),
		}
	}
}

impl LayoutConstraints {
	pub fn set_width(&mut self, width: f32) {
		self.min_width.set(width);
		self.max_width.set(width);
	}

	pub fn set_height(&mut self, height: f32) {
		self.min_height.set(height);
		self.max_height.set(height);
	}

	pub fn set_size(&mut self, size: Vec2) {
		self.set_width(size.x);
		self.set_height(size.y);
	}
}


impl LayoutConstraints {
	pub fn min_width(&self) -> f32 {
		let min_width = self.min_width.get().max(self.padding.horizontal_sum());

		if self.horizontal_size_policy.get().contains(SizingBehaviour::CAN_SHRINK) {
			min_width
		} else {
			self.preferred_width()
		}
	}

	pub fn max_width(&self) -> f32 {
		let max_width = self.max_width.get().max(self.padding.horizontal_sum());

		if self.horizontal_size_policy.get().contains(SizingBehaviour::CAN_GROW) {
			max_width
		} else {
			self.preferred_width()
		}
	}

	pub fn min_height(&self) -> f32 {
		let min_height = self.min_height.get().max(self.padding.vertical_sum());

		if self.vertical_size_policy.get().contains(SizingBehaviour::CAN_SHRINK) {
			min_height
		} else {
			self.preferred_height()
		}
	}

	pub fn max_height(&self) -> f32 {
		let max_height = self.max_height.get().max(self.padding.vertical_sum());

		if self.vertical_size_policy.get().contains(SizingBehaviour::CAN_GROW) {
			max_height
		} else {
			self.preferred_height()
		}
	}

	pub fn preferred_width(&self) -> f32 {
		self.preferred_width.get()
			.clamp(self.min_width.get(), self.max_width.get())
			.max(self.padding.horizontal_sum())
	}

	pub fn preferred_height(&self) -> f32 {
		self.preferred_height.get()
			.clamp(self.min_height.get(), self.max_height.get())
			.max(self.padding.vertical_sum())
	}
}





pub fn layout_children(
	available_bounds: Aabb2,
	widgets: &[WidgetId],
	constraints: &SecondaryMap<WidgetId, LayoutConstraints>,
	layouts: &mut SecondaryMap<WidgetId, Layout>)
{
	if widgets.is_empty() {
		return;
	}

	size_layouts(available_bounds, widgets, constraints, layouts);
	position_layouts(available_bounds, widgets, constraints, layouts);
}


fn size_layouts(available_bounds: Aabb2,
	widgets: &[WidgetId],
	constraints: &SecondaryMap<WidgetId, LayoutConstraints>,
	layouts: &mut SecondaryMap<WidgetId, Layout>)
{
	let available_width = available_bounds.width();

	let mut total_min_width = 0.0f32;
	let mut total_preferred_width = 0.0f32;
	let mut total_max_width = 0.0f32;
	let mut total_margins = 0.0f32;

	let mut num_unconstrained_growable_widgets = 0;

	for &widget_id in widgets {
		let constraints = &constraints[widget_id];

		let margin = constraints.margin.horizontal_sum();

		let min_width = constraints.min_width().max(0.0);
		let preferred_width = constraints.preferred_width();
		let max_width = constraints.max_width();

		if max_width.is_finite() {
			total_max_width += max_width;
		} else {
			num_unconstrained_growable_widgets += 1;
		}

		total_min_width += min_width;
		total_preferred_width += preferred_width;
		total_margins += margin;

		layouts.insert(widget_id, Layout::default());
	}

	let max_constrained_width = available_width - total_margins;
	total_max_width += num_unconstrained_growable_widgets as f32 * max_constrained_width;



	if available_width <= total_min_width + total_margins {
		// Every widget takes min_size
		for &widget_id in widgets {
			let constraints = &constraints[widget_id];
			let layout = &mut layouts[widget_id];

			layout.size.x = constraints.min_width();
			layout.size.y = constraints.preferred_height();
		}

		return;
	}

	// available_width > total_min_width + total_margins 

	if available_width <= total_preferred_width + total_margins {
		let total_weight = total_preferred_width - total_min_width;
		let spare_space = available_width - total_margins - total_min_width;

		// Every widget shrinks proportionally from preferred_size to min_size
		for &widget_id in widgets {
			let constraints = &constraints[widget_id];
			let layout = &mut layouts[widget_id];

			let weight = constraints.preferred_width() - constraints.min_width();
			let extra_width = weight / total_weight * spare_space;

			layout.size.x = constraints.min_width() + extra_width;
			layout.size.y = constraints.preferred_height();
		}

		return;
	}

	// available_width > total_preferred_width + total_margins
	let total_weight = total_max_width - total_preferred_width;
	let spare_space = available_width - total_margins - total_preferred_width;

	for &widget_id in widgets {
		let constraints = &constraints[widget_id];
		let layout = &mut layouts[widget_id];

		let max_width = constraints.max_width();
		let max_width_constrained = max_width.min(max_constrained_width);

		let weight = max_width_constrained - constraints.preferred_width();
		let extra_width = if total_weight > 0.0 {
			weight / total_weight * spare_space
		} else {
			0.0
		};

		layout.size.x = (constraints.preferred_width() + extra_width.max(0.0)).min(max_width);
		layout.size.y = constraints.preferred_height();
	}
}


fn position_layouts(available_bounds: Aabb2,
	widgets: &[WidgetId],
	constraints: &SecondaryMap<WidgetId, LayoutConstraints>,
	layouts: &mut SecondaryMap<WidgetId, Layout>)
{
	let mut cursor = available_bounds.min_max_corner();

	for &widget_id in widgets {
		let constraints = &constraints[widget_id];
		let layout = &mut layouts[widget_id];
		let margin_box_tl = cursor + Vec2::new(constraints.margin.left.get(), -constraints.margin.top.get());
		let min_pos = margin_box_tl - Vec2::from_y(layout.size.y);
		let max_pos = margin_box_tl + Vec2::from_x(layout.size.x);

		cursor.x = max_pos.x + constraints.margin.right.get();

		let box_bounds = Aabb2::new(min_pos, max_pos);
		let content_bounds = inset_lengths(&box_bounds, &constraints.padding);
		let margin_bounds = outset_lengths(&box_bounds, &constraints.margin);

		layout.box_bounds = box_bounds;
		layout.content_bounds = content_bounds;
		layout.margin_bounds = margin_bounds;
	}
}




pub fn inset_lengths(bounds: &Aabb2, lengths: &BoxLengths) -> Aabb2 {
	let min = bounds.min + Vec2::new(lengths.left.get(), lengths.bottom.get());
	let max = bounds.max - Vec2::new(lengths.right.get(), lengths.top.get());

	Aabb2::new(
		Vec2::new(min.x.min(max.x), min.y.min(max.y)),
		Vec2::new(min.x.max(max.x), min.y.max(max.y))
	)
}

pub fn outset_lengths(bounds: &Aabb2, lengths: &BoxLengths) -> Aabb2 {
	let min = bounds.min - Vec2::new(lengths.left.get(), lengths.bottom.get());
	let max = bounds.max + Vec2::new(lengths.right.get(), lengths.top.get());

	Aabb2::new(
		Vec2::new(min.x.min(max.x), min.y.min(max.y)),
		Vec2::new(min.x.max(max.x), min.y.max(max.y))
	)
}



#[derive(Debug, Clone)]
pub struct Layout {
	pub size: Vec2,

	pub box_bounds: Aabb2,
	pub margin_bounds: Aabb2,
	pub content_bounds: Aabb2,
}

impl Default for Layout {
	fn default() -> Self {
		Layout {
			size: Vec2::zero(),

			box_bounds: Aabb2::zero(),
			margin_bounds: Aabb2::zero(),
			content_bounds: Aabb2::zero(),
		}
	}
}