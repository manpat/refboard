use crate::prelude::*;

use lyon::math::{Point, Box2D};
use lyon::path::Path;
use lyon::tessellation::*;


pub struct Painter {
	pub geometry: VertexBuffers<renderer::Vertex, u32>,

	fill_tess: FillTessellator,
	stroke_tess: StrokeTessellator,
	fill_options: FillOptions,
	stroke_options: StrokeOptions,
}

impl Painter {
	pub fn new() -> Painter {
		Painter {
			geometry: VertexBuffers::new(),
			fill_tess: FillTessellator::new(),
			stroke_tess: StrokeTessellator::new(),

			fill_options: FillOptions::tolerance(0.02).with_fill_rule(FillRule::NonZero),
			stroke_options: StrokeOptions::DEFAULT,
		}
	}

	pub fn clear(&mut self) {
		self.geometry.vertices.clear();
		self.geometry.indices.clear();
	}

	fn geo_builder<'g>(geo: &'g mut VertexBuffers<renderer::Vertex, u32>, color: impl Into<Color>) -> (impl StrokeGeometryBuilder + FillGeometryBuilder + 'g) {
		let color = color.into().to_array();
		BuffersBuilder::new(geo, VertexConstructor { color }).with_inverted_winding()
	}
}

impl Painter {
	pub fn circle(&mut self, pos: impl Into<Vec2>, r: f32, color: impl Into<Color>) {
		let pos = to_point(pos.into());
		self.stroke_tess.tessellate_circle(pos, r, &self.stroke_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn fill_circle(&mut self, pos: impl Into<Vec2>, r: f32, color: impl Into<Color>) {
		let pos = to_point(pos.into());
		self.fill_tess.tessellate_circle(pos, r, &self.fill_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn rect(&mut self, rect: impl Into<Aabb2>, color: impl Into<Color>) {
		let rect = to_box(rect.into());
		self.stroke_tess.tessellate_rectangle(&rect, &self.stroke_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn fill_rect(&mut self, rect: impl Into<Aabb2>, color: impl Into<Color>) {
		let rect = to_box(rect.into());
		self.fill_tess.tessellate_rectangle(&rect, &self.fill_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn stroke_path(&mut self, path: &Path, color: impl Into<Color>) {
		self.stroke_tess.tessellate_path(path, &self.stroke_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn fill_path(&mut self, path: &Path, color: impl Into<Color>) {
		self.fill_tess.tessellate_path(path, &self.fill_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}
}




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
	Box2D::new(to_point(min), to_point(max))
}