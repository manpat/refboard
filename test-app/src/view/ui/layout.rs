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

// #[derive(Debug, Clone, Copy)]
// pub enum SizingBehaviour {
// 	Normal,
// }


#[derive(Debug, Clone)]
pub struct LayoutConstraints {
	pub min_width: WidgetParameter<f32>,
	pub min_height: WidgetParameter<f32>,

	pub max_width: WidgetParameter<f32>,
	pub max_height: WidgetParameter<f32>,

	pub preferred_width: WidgetParameter<f32>,
	pub preferred_height: WidgetParameter<f32>,

	pub margin: BoxLengths,
	pub padding: BoxLengths,

	// pub horizontal_size_policy: WidgetParameter<SizingBehaviour>,
	// pub vertical_size_policy: WidgetParameter<SizingBehaviour>,

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

			// horizontal_size_policy: WidgetParameter::new(SizePolicy::PREFERRED),
			// vertical_size_policy: WidgetParameter::new(SizePolicy::PREFERRED),
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
	pub fn desired_width(&self) -> f32 {
		let padding_total = self.padding.horizontal_sum();
		(self.preferred_width.get() + padding_total).clamp(self.min_width.get(), self.max_width.get()) 
	}

	pub fn desired_height(&self) -> f32 {
		let padding_total = self.padding.vertical_sum();
		(self.preferred_height.get() + padding_total).clamp(self.min_height.get(), self.max_height.get()) 
	}

	pub fn outer_min_width(&self) -> f32 {
		self.min_width.get().max(self.padding.horizontal_sum()) + self.margin.horizontal_sum()
	}

	pub fn outer_min_height(&self) -> f32 {
		self.min_height.get().max(self.padding.vertical_sum()) + self.margin.vertical_sum()
	}

	pub fn outer_desired_width(&self) -> f32 {
		self.desired_width() + self.margin.horizontal_sum()
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

	let mut total_min_width = 0.0f32;
	let mut total_preferred_width = 0.0f32;

	for &widget_id in widgets {
		let constraints = &constraints[widget_id];

		let min_width = constraints.outer_min_width();
		let preferred_width = constraints.outer_desired_width();

		total_min_width += min_width;
		total_preferred_width += preferred_width;

		layouts.insert(widget_id, Layout::default());
	}

	let available_width = available_bounds.width();
	let widget_count = widgets.len();

	let mut cursor = available_bounds.min_max_corner();

	if available_width <= total_min_width {
		// Every widget takes min_size
		for &widget_id in widgets {
			let constraints = &constraints[widget_id];

			let padding_width = constraints.min_width.get();
			let padding_height = constraints.desired_height();

			let margin_box_tl = cursor + Vec2::new(constraints.margin.left.get(), -constraints.margin.top.get());
			let min_pos = margin_box_tl - Vec2::from_y(padding_height);
			let max_pos = margin_box_tl + Vec2::from_x(padding_width);

			cursor.x = max_pos.x + constraints.margin.right.get();

			let box_bounds = Aabb2::new(min_pos, max_pos);
			let content_bounds = inset_lengths(&box_bounds, &constraints.padding);
			let margin_bounds = outset_lengths(&box_bounds, &constraints.margin);

			let layout = &mut layouts[widget_id];
			layout.box_bounds = box_bounds;
			layout.content_bounds = content_bounds;
			layout.margin_bounds = margin_bounds;
		}

		return;
	}

	if available_width <= total_preferred_width {
		let total_weight = total_preferred_width - total_min_width;
		assert!(total_weight > 0.0);

		// Every widget shrinks proportionally from preferred_size to min_size
		for &widget_id in widgets {
			let constraints = &constraints[widget_id];

			let weight = constraints.outer_desired_width() - constraints.outer_min_width();
			let extra_width = weight / total_weight * (available_width - total_min_width);

			let padding_width = constraints.min_width.get() + extra_width;
			let padding_height = constraints.desired_height();

			let margin_box_tl = cursor + Vec2::new(constraints.margin.left.get(), -constraints.margin.top.get());
			let min_pos = margin_box_tl - Vec2::from_y(padding_height);
			let max_pos = margin_box_tl + Vec2::from_x(padding_width);

			cursor.x = max_pos.x + constraints.margin.right.get();

			let box_bounds = Aabb2::new(min_pos, max_pos);
			let content_bounds = inset_lengths(&box_bounds, &constraints.padding);
			let margin_bounds = outset_lengths(&box_bounds, &constraints.margin);

			let layout = &mut layouts[widget_id];
			layout.box_bounds = box_bounds;
			layout.content_bounds = content_bounds;
			layout.margin_bounds = margin_bounds;
		}

		return;
	}

	for &widget_id in widgets {
		let constraints = &constraints[widget_id];

		let padding_width = constraints.desired_width();
		let padding_height = constraints.desired_height();

		let margin_box_tl = cursor + Vec2::new(constraints.margin.left.get(), -constraints.margin.top.get());
		let min_pos = margin_box_tl - Vec2::from_y(padding_height);
		let max_pos = margin_box_tl + Vec2::from_x(padding_width);

		cursor.x = max_pos.x + constraints.margin.right.get();

		let box_bounds = Aabb2::new(min_pos, max_pos);
		let content_bounds = inset_lengths(&box_bounds, &constraints.padding);
		let margin_bounds = outset_lengths(&box_bounds, &constraints.margin);

		let layout = &mut layouts[widget_id];
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

	pub box_bounds: Aabb2,
	pub margin_bounds: Aabb2,
	pub content_bounds: Aabb2,
}

impl Default for Layout {
	fn default() -> Self {
		Layout {
			box_bounds: Aabb2::zero(),
			margin_bounds: Aabb2::zero(),
			content_bounds: Aabb2::zero(),
		}
	}
}