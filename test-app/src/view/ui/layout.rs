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



#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Axis {
	Horizontal,
	Vertical,
}

impl Axis {
	pub fn opposite(&self) -> Self {
		match self {
			Axis::Horizontal => Axis::Vertical,
			Axis::Vertical => Axis::Horizontal,
		}
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

	pub fn set_axis(&mut self, axis: Axis, value: f32)  {
		match axis {
			Axis::Horizontal => self.set_horizontal(value),
			Axis::Vertical => self.set_vertical(value),
		}
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

	pub fn axis_sum(&self, axis: Axis) -> f32 {
		match axis {
			Axis::Horizontal => self.horizontal_sum(),
			Axis::Vertical => self.vertical_sum(),
		}
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

	// Includes padding
	pub preferred_width: WidgetParameter<f32>,
	pub preferred_height: WidgetParameter<f32>,

	pub margin: BoxLengths,
	pub padding: BoxLengths,

	pub horizontal_size_policy: WidgetParameter<SizingBehaviour>,
	pub vertical_size_policy: WidgetParameter<SizingBehaviour>,

	pub children_layout_axis: WidgetParameter<Axis>,

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

			children_layout_axis: WidgetParameter::new(Axis::Horizontal),
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
	pub fn min_length_mut(&mut self, axis: Axis) -> &mut WidgetParameter<f32> {
		match axis {
			Axis::Horizontal => &mut self.min_width,
			Axis::Vertical => &mut self.min_height,
		}
	}

	pub fn preferred_length_mut(&mut self, axis: Axis) -> &mut WidgetParameter<f32> {
		match axis {
			Axis::Horizontal => &mut self.preferred_width,
			Axis::Vertical => &mut self.preferred_height,
		}
	}

	pub fn max_length_mut(&mut self, axis: Axis) -> &mut WidgetParameter<f32> {
		match axis {
			Axis::Horizontal => &mut self.max_width,
			Axis::Vertical => &mut self.max_height,
		}
	}

	pub fn size_policy_mut(&mut self, axis: Axis) -> &mut WidgetParameter<SizingBehaviour> {
		match axis {
			Axis::Horizontal => &mut self.horizontal_size_policy,
			Axis::Vertical => &mut self.vertical_size_policy,
		}
	}


	pub fn min_length_unconstrained(&self, axis: Axis) -> f32 {
		match axis {
			Axis::Horizontal => self.min_width.get(),
			Axis::Vertical => self.min_height.get(),
		}
	}

	pub fn preferred_length_unconstrained(&self, axis: Axis) -> f32 {
		match axis {
			Axis::Horizontal => self.preferred_width.get(),
			Axis::Vertical => self.preferred_height.get(),
		}
	}

	pub fn max_length_unconstrained(&self, axis: Axis) -> f32 {
		match axis {
			Axis::Horizontal => self.max_width.get(),
			Axis::Vertical => self.max_height.get(),
		}
	}

	pub fn size_policy(&self, axis: Axis) -> SizingBehaviour {
		match axis {
			Axis::Horizontal => self.horizontal_size_policy.get(),
			Axis::Vertical => self.vertical_size_policy.get(),
		}
	}

	pub fn min_length(&self, axis: Axis) -> f32 {
		if self.size_policy(axis).contains(SizingBehaviour::CAN_SHRINK) {
			self.min_length_unconstrained(axis).max(self.padding.axis_sum(axis))
		} else {
			self.preferred_length(axis)
		}
	}

	pub fn max_length(&self, axis: Axis) -> f32 {
		if self.size_policy(axis).contains(SizingBehaviour::CAN_GROW) {
			self.max_length_unconstrained(axis).max(self.padding.axis_sum(axis))
		} else {
			self.preferred_length(axis)
		}
	}

	pub fn preferred_length(&self, axis: Axis) -> f32 {
		self.preferred_length_unconstrained(axis)
			.clamp(self.min_length_unconstrained(axis), self.max_length_unconstrained(axis))
			.max(self.padding.axis_sum(axis))
	}
}





pub fn layout_children_linear(
	available_bounds: Aabb2,
	main_axis: Axis,
	widgets: &[WidgetId],
	constraints: &SecondaryMap<WidgetId, LayoutConstraints>,
	layouts: &mut SecondaryMap<WidgetId, Layout>)
{
	if widgets.is_empty() {
		return;
	}

	// TODO(pat.m): pls use slotmaps less - this is unnecessary
	for &widget_id in widgets {
		layouts.insert(widget_id, Layout::default());
	}

	let cross_axis = main_axis.opposite();

	size_layouts_linear(available_bounds, main_axis, widgets, constraints, layouts);
	size_layouts_overlapping(available_bounds, cross_axis, widgets, constraints, layouts);

	position_layouts_linear(available_bounds, main_axis, widgets, constraints, layouts);
	// TODO(pat.m): position_layouts_overlapping - for alignment
}

fn size_layouts_linear(available_bounds: Aabb2,
	main_axis: Axis,
	widgets: &[WidgetId],
	constraints: &SecondaryMap<WidgetId, LayoutConstraints>,
	layouts: &mut SecondaryMap<WidgetId, Layout>)
{
	let mut available_length = match main_axis {
		Axis::Horizontal => available_bounds.width(),
		Axis::Vertical => available_bounds.height(),
	};

	let mut remaining_widgets = widgets.len();

	for &widget_id in widgets {
		let constraints = &constraints[widget_id];
		available_length -= constraints.margin.axis_sum(main_axis);
		available_length -= constraints.min_length(main_axis);
	}

	'next_pass: loop {
		let allocated_extra_length = (available_length / remaining_widgets as f32).max(0.0);

		'current_pass: for &widget_id in widgets {
			let layout = &mut layouts[widget_id];
			if layout.final_size {
				continue 'current_pass;
			}

			let constraints = &constraints[widget_id];

			let min_length = constraints.min_length(main_axis);
			let max_length = constraints.max_length(main_axis);
			let extra_length = max_length - min_length;

			let layout_length = match main_axis {
				Axis::Horizontal => &mut layout.size.x,
				Axis::Vertical => &mut layout.size.y,
			};

			if allocated_extra_length >= extra_length {
				*layout_length = min_length + extra_length;
				layout.final_size = true;
				remaining_widgets -= 1;
				available_length -= extra_length;
				continue 'next_pass;

			} else {
				let extra_length = allocated_extra_length.min(extra_length);
				*layout_length = min_length + extra_length;
			}
		}

		break
	}
}

fn size_layouts_overlapping(available_bounds: Aabb2,
	main_axis: Axis,
	widgets: &[WidgetId],
	constraints: &SecondaryMap<WidgetId, LayoutConstraints>,
	layouts: &mut SecondaryMap<WidgetId, Layout>)
{
	let available_length = match main_axis {
		Axis::Horizontal => available_bounds.width(),
		Axis::Vertical => available_bounds.height(),
	};

	for &widget_id in widgets {
		let layout = &mut layouts[widget_id];
		let constraints = &constraints[widget_id];

		let margin = constraints.margin.axis_sum(main_axis);

		let min_length = constraints.min_length(main_axis);
		let max_length = constraints.max_length(main_axis);

		let layout_length = match main_axis {
			Axis::Horizontal => &mut layout.size.x,
			Axis::Vertical => &mut layout.size.y,
		};

		*layout_length = (available_length - margin).clamp(min_length, max_length);
	}
}


fn position_layouts_linear(available_bounds: Aabb2,
	main_axis: Axis,
	widgets: &[WidgetId],
	constraints: &SecondaryMap<WidgetId, LayoutConstraints>,
	layouts: &mut SecondaryMap<WidgetId, Layout>)
{
	let mut cursor = available_bounds.min;

	for &widget_id in widgets {
		let constraints = &constraints[widget_id];
		let layout = &mut layouts[widget_id];

		let min_pos = cursor + Vec2::new(constraints.margin.left.get(), constraints.margin.top.get());
		let max_pos = min_pos + layout.size;

		match main_axis {
			Axis::Horizontal => cursor.x = max_pos.x + constraints.margin.right.get(),
			Axis::Vertical => cursor.y = max_pos.y + constraints.margin.bottom.get(),
		}

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
	pub final_size: bool,

	pub box_bounds: Aabb2,
	pub margin_bounds: Aabb2,
	pub content_bounds: Aabb2,
}

impl Default for Layout {
	fn default() -> Self {
		Layout {
			size: Vec2::zero(),
			final_size: false,

			box_bounds: Aabb2::zero(),
			margin_bounds: Aabb2::zero(),
			content_bounds: Aabb2::zero(),
		}
	}
}