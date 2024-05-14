use crate::prelude::*;

use lyon::math::{Point, Box2D};
use lyon::path::{Path, PathBuffer, Winding as PathWinding};
use lyon::path::builder::{PathBuilder, BorderRadii};
use lyon::tessellation::*;


pub struct Painter {
	pub geometry: VertexBuffers<renderer::Vertex, u32>,

	fill_tess: FillTessellator,
	stroke_tess: StrokeTessellator,
	fill_options: FillOptions,
	stroke_options: StrokeOptions,

	scratch_path: PathBuffer,
}

impl Painter {
	pub fn new() -> Painter {
		Painter {
			geometry: VertexBuffers::new(),
			fill_tess: FillTessellator::new(),
			stroke_tess: StrokeTessellator::new(),

			fill_options: FillOptions::tolerance(0.1).with_fill_rule(FillRule::NonZero),
			stroke_options: StrokeOptions::tolerance(0.1),

			scratch_path: PathBuffer::new(),
		}
	}

	pub fn clear(&mut self) {
		self.geometry.vertices.clear();
		self.geometry.indices.clear();

		self.fill_options = FillOptions::tolerance(0.5).with_fill_rule(FillRule::NonZero);
		self.stroke_options = StrokeOptions::tolerance(0.5);
	}

	fn geo_builder<'g>(geo: &'g mut VertexBuffers<renderer::Vertex, u32>, color: impl Into<Color>) -> (impl StrokeGeometryBuilder + FillGeometryBuilder + 'g) {
		let color = color.into().to_array();
		BuffersBuilder::new(geo, VertexConstructor { color })
	}
}

impl Painter {
	pub fn set_line_width(&mut self, width: f32) {
		self.stroke_options.line_width = width;
	}
}

impl Painter {
	pub fn circle_outline(&mut self, pos: impl Into<Vec2>, r: f32, color: impl Into<Color>) {
		let pos = to_point(pos.into());
		self.stroke_tess.tessellate_circle(pos, r, &self.stroke_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn circle(&mut self, pos: impl Into<Vec2>, r: f32, color: impl Into<Color>) {
		let pos = to_point(pos.into());
		self.fill_tess.tessellate_circle(pos, r, &self.fill_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn rect_outline(&mut self, rect: impl Into<Aabb2>, color: impl Into<Color>) {
		let rect = to_box(rect.into());
		self.stroke_tess.tessellate_rectangle(&rect, &self.stroke_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn rect(&mut self, rect: impl Into<Aabb2>, color: impl Into<Color>) {
		let rect = to_box(rect.into());
		self.fill_tess.tessellate_rectangle(&rect, &self.fill_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn rounded_rect_outline(&mut self, rect: impl Into<Aabb2>, radii: impl IntoBorderRadii, color: impl Into<Color>) {
		let rect = to_box(rect.into());

		self.scratch_path.clear();

		let mut builder = self.scratch_path.builder();
		builder.add_rounded_rectangle(&rect, &radii.to_radii(), PathWinding::Positive, &[]);
		builder.build();

		self.stroke_tess.tessellate(self.scratch_path.get(0).iter(), &self.stroke_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn rounded_rect(&mut self, rect: impl Into<Aabb2>, radii: impl IntoBorderRadii, color: impl Into<Color>) {
		let rect = to_box(rect.into());

		self.scratch_path.clear();

		let mut builder = self.scratch_path.builder();
		builder.add_rounded_rectangle(&rect, &radii.to_radii(), PathWinding::Positive, &[]);
		builder.build();

		self.fill_tess.tessellate(self.scratch_path.get(0).iter(), &self.fill_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn stroke_path(&mut self, path: &Path, color: impl Into<Color>) {
		self.stroke_tess.tessellate_path(path, &self.stroke_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn fill_path(&mut self, path: &Path, color: impl Into<Color>) {
		self.fill_tess.tessellate_path(path, &self.fill_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn line(&mut self, start: impl Into<Vec2>, end: impl Into<Vec2>, color: impl Into<Color>) {
		use lyon::path::PathEvent;

		let start = to_point(start.into());
		let end = to_point(end.into());

		let events = [
			PathEvent::Begin{
				at: start
			},
			PathEvent::Line{
				from: start,
				to: end,
			},
			PathEvent::End{
				last: end,
				first: start,
				close: false,
			},
		];

		self.stroke_tess.tessellate(events, &self.stroke_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}
}



pub trait IntoBorderRadii {
	fn to_radii(&self) -> BorderRadii;
}

impl IntoBorderRadii for f32 {
	fn to_radii(&self) -> BorderRadii {
		BorderRadii::new(*self)
	}
}

impl IntoBorderRadii for BorderRadii {
	fn to_radii(&self) -> BorderRadii {
		*self
	}
}

// TODO(pat.m): IntoBorderRadii for type that can represent which corners should be rounded



struct VertexConstructor {
	color: [f32; 4],

}

impl FillVertexConstructor<renderer::Vertex> for VertexConstructor {
	fn new_vertex(&mut self, vertex: FillVertex) -> renderer::Vertex {
		let pos = vertex.position().to_array();
		renderer::Vertex {
			pos,
			color: self.color,
		}
	}
}

impl StrokeVertexConstructor<renderer::Vertex> for VertexConstructor {
	fn new_vertex(&mut self, vertex: StrokeVertex) -> renderer::Vertex {
		let pos = vertex.position().to_array();
		renderer::Vertex {
			pos,
			color: self.color,
		}
	}
}



fn to_point(Vec2{x,y}: Vec2) -> Point {
	Point::new(x, y)
}

fn to_box(Aabb2{min, max}: Aabb2) -> Box2D {
	// TODO(pat.m): make this more intentional
	// Box2D::new(to_point(min.ceil() - Vec2::splat(0.5)), to_point(max.floor() + Vec2::splat(0.5)))
	Box2D::new(to_point(min.floor() + Vec2::splat(0.5)), to_point(max.floor() - Vec2::splat(0.5)))
}