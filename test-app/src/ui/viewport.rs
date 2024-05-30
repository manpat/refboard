use crate::prelude::*;

#[derive(Debug, Default)]
pub struct Viewport {
	pub size: Vec2,
}

impl Viewport {
	pub fn new() -> Viewport {
		Viewport {
			size: Vec2::zero(),
		}
	}

	pub fn view_to_clip(&self) -> Mat2x3 {
		Mat2x3::scale_translate(Vec2::new(2.0, -2.0) / self.size, Vec2::new(-1.0, 1.0))
	}

	pub fn physical_to_view(&self) -> Mat2x3 {
		Mat2x3::identity()
	}

	pub fn view_bounds(&self) -> Aabb2 {
		Aabb2::new(Vec2::zero(), self.size)
	}
}