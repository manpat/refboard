use crate::prelude::*;
use super::{WidgetId, BoxLengths, WidgetBox};


pub type LayoutConstraintMap = SecondaryMap<WidgetId, LayoutConstraints>;

bitflags! {
	#[derive(Copy, Clone, Debug)]
	pub struct SetConstraints : u32 {
		const MIN_WIDTH = 1 << 0;
		const MIN_HEIGHT = 1 << 1;

		const MAX_WIDTH = 1 << 2;
		const MAX_HEIGHT = 1 << 3;

		const CONTENT_WIDTH = 1 << 4;
		const CONTENT_HEIGHT = 1 << 5;

		const MARGIN_TOP = 1 << 6;
		const MARGIN_BOTTOM = 1 << 7;
		const MARGIN_LEFT = 1 << 8;
		const MARGIN_RIGHT = 1 << 9;

		const ALL_MARGINS = 0b1111 << 6;

		const PADDING_TOP = 1 << 10;
		const PADDING_BOTTOM = 1 << 11;
		const PADDING_LEFT = 1 << 12;
		const PADDING_RIGHT = 1 << 13;

		const ALL_PADDING = 0b1111 << 10;
	}
}





#[derive(Debug, Clone)]
pub struct LayoutConstraints {
	pub min_width: f32,
	pub min_height: f32,

	pub max_width: f32,
	pub max_height: f32,

	pub content_width: f32,
	pub content_height: f32,

	pub margin: BoxLengths,
	pub padding: BoxLengths,

	// TODO(pat.m): baselines??

	// size policies
	// alignment

	// children layout policy

	pub set: SetConstraints,
}

impl Default for LayoutConstraints {
	fn default() -> Self {
		LayoutConstraints {
			min_width: 0.0,
			min_height: 0.0,

			max_width: f32::INFINITY,
			max_height: f32::INFINITY,

			content_width: 0.0,
			content_height: 0.0,

			margin: BoxLengths::default(),
			padding: BoxLengths::default(),

			set: SetConstraints::empty(),
		}
	}
}

impl LayoutConstraints {
	pub fn set_min_width(&mut self, min_width: f32) {
		self.min_width = min_width;
		self.set.insert(SetConstraints::MIN_WIDTH);
	}

	pub fn set_min_height(&mut self, min_height: f32) {
		self.min_height = min_height;
		self.set.insert(SetConstraints::MIN_HEIGHT);
	}

	pub fn set_min_size(&mut self, min_size: Vec2) {
		self.set_min_width(min_size.x);
		self.set_min_height(min_size.y);
	}

	pub fn set_max_width(&mut self, max_width: f32) {
		self.max_width = max_width;
		self.set.insert(SetConstraints::MAX_WIDTH);
	}

	pub fn set_max_height(&mut self, max_height: f32) {
		self.max_height = max_height;
		self.set.insert(SetConstraints::MAX_HEIGHT);
	}

	pub fn set_max_size(&mut self, max_size: Vec2) {
		self.set_max_width(max_size.x);
		self.set_max_height(max_size.y);
	}

	pub fn set_content_width(&mut self, content_width: f32) {
		self.content_width = content_width;
		self.set.insert(SetConstraints::CONTENT_WIDTH);
	}
	
	pub fn set_content_height(&mut self, content_height: f32) {
		self.content_height = content_height;
		self.set.insert(SetConstraints::CONTENT_HEIGHT);
	}

	pub fn set_content_size(&mut self, content_size: Vec2) {
		self.set_content_width(content_size.x);
		self.set_content_height(content_size.y);
	}


	pub fn set_width(&mut self, width: f32) {
		self.set_min_width(width);
		self.set_max_width(width);
	}
	pub fn set_height(&mut self, height: f32) {
		self.set_min_height(height);
		self.set_max_height(height);
	}
	pub fn set_size(&mut self, size: Vec2) {
		self.set_min_size(size);
		self.set_max_size(size);
	}


	pub fn set_margin_top(&mut self, top: f32) {
		self.margin.top = top;
		self.set.insert(SetConstraints::MARGIN_TOP);
	}
	
	pub fn set_margin_bottom(&mut self, bottom: f32) {
		self.margin.bottom = bottom;
		self.set.insert(SetConstraints::MARGIN_BOTTOM);
	}

	pub fn set_margin_left(&mut self, left: f32) {
		self.margin.left = left;
		self.set.insert(SetConstraints::MARGIN_LEFT);
	}
	
	pub fn set_margin_right(&mut self, right: f32) {
		self.margin.right = right;
		self.set.insert(SetConstraints::MARGIN_RIGHT);
	}

	pub fn set_horizontal_margin(&mut self, margin: f32) {
		self.set_margin_left(margin);
		self.set_margin_right(margin);
	}

	pub fn set_vertical_margin(&mut self, margin: f32) {
		self.set_margin_top(margin);
		self.set_margin_bottom(margin);
	}

	pub fn set_margin(&mut self, margin: impl Into<BoxLengths>) {
		self.margin = margin.into();
		self.set.insert(SetConstraints::ALL_MARGINS);
	}


	pub fn set_padding_top(&mut self, top: f32) {
		self.padding.top = top;
		self.set.insert(SetConstraints::PADDING_TOP);
	}
	
	pub fn set_padding_bottom(&mut self, bottom: f32) {
		self.padding.bottom = bottom;
		self.set.insert(SetConstraints::PADDING_BOTTOM);
	}

	pub fn set_padding_left(&mut self, left: f32) {
		self.padding.left = left;
		self.set.insert(SetConstraints::PADDING_LEFT);
	}
	
	pub fn set_padding_right(&mut self, right: f32) {
		self.padding.right = right;
		self.set.insert(SetConstraints::PADDING_RIGHT);
	}

	pub fn set_horizontal_padding(&mut self, padding: f32) {
		self.set_padding_left(padding);
		self.set_padding_right(padding);
	}

	pub fn set_vertical_padding(&mut self, padding: f32) {
		self.set_padding_top(padding);
		self.set_padding_bottom(padding);
	}

	pub fn set_padding(&mut self, padding: impl Into<BoxLengths>) {
		self.padding = padding.into();
		self.set.insert(SetConstraints::ALL_PADDING);
	}
}


impl LayoutConstraints {
	pub fn desired_width(&self) -> f32 {
		resolve_length(self.min_width, self.max_width, self.content_width, (self.padding.left, self.padding.right))
	}

	pub fn desired_height(&self) -> f32 {
		resolve_length(self.min_height, self.max_height, self.content_height, (self.padding.top, self.padding.bottom))
	}

	pub fn is_set(&self, constraints: SetConstraints) -> bool {
		self.set.contains(constraints)
	}

	pub fn set_default(&mut self, constraints: SetConstraints, f: impl FnOnce(&mut Self)) {
		if !self.set.contains(constraints) {
			f(self);
			self.set.insert(constraints);
		}
	}
}





pub fn layout(
	available_bounds: Aabb2,
	widgets: &[WidgetId],
	constraints: &SecondaryMap<WidgetId, LayoutConstraints>,
	widget_boxes: &mut SecondaryMap<WidgetId, WidgetBox>)
{
	let mut cursor = available_bounds.min_max_corner();
	let mut prev_margin = 0.0f32;

	for &widget_id in widgets {
		let constraints = &constraints[widget_id];

		let padding_width = constraints.desired_width();
		let padding_height = constraints.desired_height();

		let margin_advance = prev_margin.max(constraints.margin.left);
		prev_margin = constraints.margin.right;

		let margin_box_tl = cursor + Vec2::new(margin_advance, -constraints.margin.top);
		let min_pos = margin_box_tl - Vec2::from_y(padding_height);
		let max_pos = margin_box_tl + Vec2::from_x(padding_width);

		cursor.x = max_pos.x;

		let box_bounds = Aabb2::new(min_pos, max_pos);
		let content_bounds = inset_lengths(&box_bounds, &constraints.padding);
		let margin_bounds = outset_lengths(&box_bounds, &constraints.margin);

		widget_boxes.insert(widget_id, WidgetBox {
			box_bounds,
			content_bounds,
			margin_bounds,
		});
	}
}


fn resolve_length(min: f32, max: f32, content: f32, padding: (f32, f32)) -> f32 {
	let padding_total = padding.0 + padding.1;
	(content + padding_total).clamp(min, max) 
}


fn inset_lengths(bounds: &Aabb2, lengths: &BoxLengths) -> Aabb2 {
	let min = bounds.min + Vec2::new(lengths.left, lengths.bottom);
	let max = bounds.max - Vec2::new(lengths.right, lengths.top);

	Aabb2::new(
		Vec2::new(min.x.min(max.x), min.y.min(max.y)),
		Vec2::new(min.x.max(max.x), min.y.max(max.y))
	)
}

fn outset_lengths(bounds: &Aabb2, lengths: &BoxLengths) -> Aabb2 {
	let min = bounds.min - Vec2::new(lengths.left, lengths.bottom);
	let max = bounds.max + Vec2::new(lengths.right, lengths.top);

	Aabb2::new(
		Vec2::new(min.x.min(max.x), min.y.min(max.y)),
		Vec2::new(min.x.max(max.x), min.y.max(max.y))
	)
}