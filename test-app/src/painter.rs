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

	clip_rect: [u16; 4],
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

			clip_rect: [0, u16::MAX, 0, u16::MAX],
		}
	}

	pub fn clear(&mut self) {
		self.geometry.vertices.clear();
		self.geometry.indices.clear();

		self.fill_options = FillOptions::tolerance(0.1).with_fill_rule(FillRule::NonZero);
		self.stroke_options = StrokeOptions::tolerance(0.1);

		self.clip_rect = [0, u16::MAX, 0, u16::MAX];
	}

	fn with_stroker(&mut self, color: impl Into<Color>, f: impl FnOnce(&mut StrokeTessellator, &StrokeOptions, &mut BuffersBuilder<'_, renderer::Vertex, u32, VertexConstructor>))
	{
		let color = color.into().to_array();
		f(&mut self.stroke_tess, &self.stroke_options,
			&mut BuffersBuilder::new(&mut self.geometry, VertexConstructor { color, clip_rect: self.clip_rect }) );
	}

	fn with_filler(&mut self, color: impl Into<Color>, f: impl FnOnce(&mut FillTessellator, &FillOptions, &mut BuffersBuilder<'_, renderer::Vertex, u32, VertexConstructor>))
	{
		let color = color.into().to_array();
		f(&mut self.fill_tess, &self.fill_options,
			&mut BuffersBuilder::new(&mut self.geometry, VertexConstructor { color, clip_rect: self.clip_rect }) );
	}
}

impl Painter {
	pub fn set_line_width(&mut self, width: f32) {
		self.stroke_options.line_width = width;
	}

	pub fn set_clip_rect(&mut self, clip_rect: [u16; 4]) {
		self.clip_rect = clip_rect;
	}
}

impl Painter {
	pub fn circle_outline(&mut self, pos: impl Into<Vec2>, r: f32, color: impl Into<Color>) {
		let pos = to_point(pos.into());
		self.with_stroker(color, |tess, opts, builder| tess.tessellate_circle(pos, r, opts, builder).unwrap());
	}

	pub fn circle(&mut self, pos: impl Into<Vec2>, r: f32, color: impl Into<Color>) {
		let pos = to_point(pos.into());
		self.with_filler(color, |tess, opts, builder| tess.tessellate_circle(pos, r, opts, builder).unwrap());
	}

	pub fn rect_outline(&mut self, rect: impl Into<Aabb2>, color: impl Into<Color>) {
		let rect = to_box_centroid(rect.into());
		self.with_stroker(color, |tess, opts, builder| tess.tessellate_rectangle(&rect, opts, builder).unwrap());
	}

	pub fn rect(&mut self, rect: impl Into<Aabb2>, color: impl Into<Color>) {
		let rect = to_box(rect.into());
		self.with_filler(color, |tess, opts, builder| tess.tessellate_rectangle(&rect, opts, builder).unwrap());
	}

	pub fn rounded_rect_outline(&mut self, rect: impl Into<Aabb2>, radii: impl IntoBorderRadii, color: impl Into<Color>) {
		let rect = to_box_centroid(rect.into());

		let mut scratch_path = std::mem::take(&mut self.scratch_path);
		scratch_path.clear();

		let mut builder = scratch_path.builder();
		builder.add_rounded_rectangle(&rect, &radii.to_radii(), PathWinding::Positive, &[]);
		builder.build();

		self.with_stroker(color, |tess, opts, builder| tess.tessellate(scratch_path.get(0).iter(), opts, builder).unwrap());
		self.scratch_path = scratch_path;
	}

	pub fn rounded_rect(&mut self, rect: impl Into<Aabb2>, radii: impl IntoBorderRadii, color: impl Into<Color>) {
		let rect = to_box(rect.into());

		let mut scratch_path = std::mem::take(&mut self.scratch_path);
		scratch_path.clear();

		let mut builder = scratch_path.builder();
		builder.add_rounded_rectangle(&rect, &radii.to_radii(), PathWinding::Positive, &[]);
		builder.build();

		self.with_filler(color, |tess, opts, builder| tess.tessellate(scratch_path.get(0).iter(), opts, builder).unwrap());
		self.scratch_path = scratch_path;
	}

	pub fn stroke_path(&mut self, path: &Path, color: impl Into<Color>) {
		self.with_stroker(color, |tess, opts, builder| tess.tessellate_path(path, opts, builder).unwrap());
	}

	pub fn fill_path(&mut self, path: &Path, color: impl Into<Color>) {
		self.with_filler(color, |tess, opts, builder| tess.tessellate_path(path, opts, builder).unwrap());
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

		self.with_stroker(color, move |tess, opts, builder| tess.tessellate(events, opts, builder).unwrap());
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
	clip_rect: [u16; 4],
}

impl FillVertexConstructor<renderer::Vertex> for VertexConstructor {
	fn new_vertex(&mut self, vertex: FillVertex) -> renderer::Vertex {
		let pos = vertex.position().to_array();
		renderer::Vertex {
			pos,
			color: self.color,
			clip_rect: self.clip_rect,
		}
	}
}

impl StrokeVertexConstructor<renderer::Vertex> for VertexConstructor {
	fn new_vertex(&mut self, vertex: StrokeVertex) -> renderer::Vertex {
		let pos = vertex.position().to_array();
		renderer::Vertex {
			pos,
			color: self.color,
			clip_rect: self.clip_rect,
		}
	}
}



fn to_point(Vec2{x,y}: Vec2) -> Point {
	Point::new(x, y)
}

fn to_box(Aabb2{min, max}: Aabb2) -> Box2D {
	// TODO(pat.m): make this more intentional
	// Box2D::new(to_point(min.ceil() - Vec2::splat(0.5)), to_point(max.floor() + Vec2::splat(0.5)))
	// Box2D::new(to_point(min.floor() + Vec2::splat(0.5)), to_point(max.floor() - Vec2::splat(0.5)))
	Box2D::new(to_point((min + Vec2::splat(0.5)).floor()), to_point((max + Vec2::splat(0.5)).floor()))
}

fn to_box_centroid(Aabb2{min, max}: Aabb2) -> Box2D {
	Box2D::new(to_point(min.floor() + Vec2::splat(0.5)), to_point(max.floor() - Vec2::splat(0.5)))
}