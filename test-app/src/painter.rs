use crate::prelude::*;

use lyon::math::{Point, Box2D};
use lyon::path::{Path, PathBuffer, Winding as PathWinding};
use lyon::path::builder::PathBuilder;
use lyon::tessellation::*;

pub use lyon::path::builder::BorderRadii;


pub struct Painter {
	pub geometry: VertexBuffers<renderer::Vertex, u32>,

	fill_tess: FillTessellator,
	stroke_tess: StrokeTessellator,

	pub fill_options: FillOptions,
	pub stroke_options: StrokeOptions,

	scratch_path: PathBuffer,

	color: Color,
	uv_rect: Option<Aabb2>,
	clip_rect: Option<Aabb2>,
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

			color: Color::white(),
			uv_rect: None,
			clip_rect: None,
		}
	}

	pub fn clear(&mut self) {
		self.geometry.vertices.clear();
		self.geometry.indices.clear();

		self.fill_options = FillOptions::tolerance(0.1).with_fill_rule(FillRule::NonZero);
		self.stroke_options = StrokeOptions::tolerance(0.1);

		self.color = Color::white();
		self.uv_rect = None;
		self.clip_rect = None;
	}

	fn with_stroker(&mut self, f: impl FnOnce(&mut StrokeTessellator, &StrokeOptions, &mut BuffersBuilder<'_, renderer::Vertex, u32, VertexConstructor>))
	{
		let num_initial_verts = self.geometry.vertices.len();
		let mut bounds = Aabb2::empty();

		let mut vertex_constructor = VertexConstructor::new(self.color, to_4u16(self.clip_rect));
		if self.uv_rect.is_some() {
			vertex_constructor.total_rect = Some(&mut bounds);
		}

		f(&mut self.stroke_tess, &self.stroke_options, &mut BuffersBuilder::new(&mut self.geometry, vertex_constructor));

		if let Some(dst_rect) = self.uv_rect {
			let src_pos = bounds.min;
			let src_size = bounds.size();

			let dst_pos = dst_rect.min;
			let dst_size = dst_rect.size();

			for vertex in &mut self.geometry.vertices[num_initial_verts..] {
				let normalised = (Vec2::from(vertex.pos) - src_pos) / src_size;
				let uv = normalised * dst_size + dst_pos;
				vertex.uv = uv.into();
			}
		}
	}

	fn with_filler(&mut self, f: impl FnOnce(&mut FillTessellator, &FillOptions, &mut BuffersBuilder<'_, renderer::Vertex, u32, VertexConstructor>))
	{
		let num_initial_verts = self.geometry.vertices.len();
		let mut bounds = Aabb2::empty();

		let mut vertex_constructor = VertexConstructor::new(self.color, to_4u16(self.clip_rect));
		if self.uv_rect.is_some() {
			vertex_constructor.total_rect = Some(&mut bounds);
		}

		f(&mut self.fill_tess, &self.fill_options, &mut BuffersBuilder::new(&mut self.geometry, vertex_constructor));

		// Fix up uvs if necessary
		if let Some(dst_rect) = self.uv_rect {
			let src_pos = bounds.min;
			let src_size = bounds.size();

			let dst_pos = dst_rect.min;
			let dst_size = dst_rect.size();

			let scale = dst_size / src_size;
			let offset = dst_pos - src_pos * scale;

			for vertex in &mut self.geometry.vertices[num_initial_verts..] {
				let uv = Vec2::from(vertex.pos) * scale + offset;
				vertex.uv = uv.into();
			}
		}
	}
}

impl Painter {
	pub fn set_line_width(&mut self, width: f32) {
		self.stroke_options.line_width = width;
	}

	pub fn set_clip_rect(&mut self, clip_rect: impl Into<Option<Aabb2>>) {
		self.clip_rect = clip_rect.into();
	}

	pub fn set_color(&mut self, color: impl Into<Color>) {
		self.color = color.into();
	}

	pub fn set_uv_rect(&mut self, uv_rect: impl Into<Option<Aabb2>>) {
		self.uv_rect = uv_rect.into();
	}
}

impl Painter {
	pub fn circle_outline(&mut self, pos: impl Into<Vec2>, r: f32) {
		let pos = to_point(pos.into());
		self.with_stroker(|tess, opts, builder| tess.tessellate_circle(pos, r, opts, builder).unwrap());
	}

	pub fn circle(&mut self, pos: impl Into<Vec2>, r: f32) {
		let pos = to_point(pos.into());
		self.with_filler(|tess, opts, builder| tess.tessellate_circle(pos, r, opts, builder).unwrap());
	}

	pub fn rect_outline(&mut self, rect: impl Into<Aabb2>) {
		let rect = to_box_centroid(rect.into());
		self.with_stroker(|tess, opts, builder| tess.tessellate_rectangle(&rect, opts, builder).unwrap());
	}

	pub fn rect(&mut self, rect: impl Into<Aabb2>) {
		let rect = to_box(rect.into());
		self.with_filler(|tess, opts, builder| tess.tessellate_rectangle(&rect, opts, builder).unwrap());
	}

	pub fn rounded_rect_outline(&mut self, rect: impl Into<Aabb2>, radii: impl IntoBorderRadii) {
		let radii = radii.to_radii();

		if radii.eq(&BorderRadii::new(0.0)) {
			self.rect_outline(rect);
			return
		}

		let rect = to_box_centroid(rect.into());

		let mut scratch_path = std::mem::take(&mut self.scratch_path);
		scratch_path.clear();

		let mut builder = scratch_path.builder();
		builder.add_rounded_rectangle(&rect, &radii, PathWinding::Positive, &[]);
		builder.build();

		self.with_stroker(|tess, opts, builder| tess.tessellate(scratch_path.get(0).iter(), opts, builder).unwrap());
		self.scratch_path = scratch_path;
	}

	pub fn rounded_rect(&mut self, rect: impl Into<Aabb2>, radii: impl IntoBorderRadii) {
		let radii = radii.to_radii();

		if radii.eq(&BorderRadii::new(0.0)) {
			self.rect(rect);
			return
		}

		let rect = to_box(rect.into());

		let mut scratch_path = std::mem::take(&mut self.scratch_path);
		scratch_path.clear();

		let mut builder = scratch_path.builder();
		builder.add_rounded_rectangle(&rect, &radii, PathWinding::Positive, &[]);
		builder.build();

		self.with_filler(|tess, opts, builder| tess.tessellate(scratch_path.get(0).iter(), opts, builder).unwrap());
		self.scratch_path = scratch_path;
	}

	pub fn stroke_path(&mut self, path: &Path) {
		self.with_stroker(|tess, opts, builder| tess.tessellate_path(path, opts, builder).unwrap());
	}

	pub fn fill_path(&mut self, path: &Path) {
		self.with_filler(|tess, opts, builder| tess.tessellate_path(path, opts, builder).unwrap());
	}

	pub fn line(&mut self, start: impl Into<Vec2>, end: impl Into<Vec2>) {
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

		self.with_stroker(move |tess, opts, builder| tess.tessellate(events, opts, builder).unwrap());
	}
}


impl Painter {
	pub fn draw_text_buffer(&mut self, buffer: &cosmic_text::Buffer, atlas: &mut ui::TextAtlas, start_pos: impl Into<Vec2>) {
		let text_color = self.color;
		let atlas_size = atlas.size().to_vec2();
		let pixel_uv_size = Vec2::one() / atlas_size;

		let start_pos = start_pos.into().to_tuple();

		for run in buffer.layout_runs() {
			for glyph in run.glyphs.iter() {
				let physical_glyph = glyph.physical(start_pos, 1.0);

				let glyph_info = atlas.request_glyph(&physical_glyph.cache_key);
				// TODO(pat.m): pass glyph_info.subpixel_alpha down to painter

				let pos = Vec2::new(physical_glyph.x as f32 + glyph_info.offset_x, physical_glyph.y as f32 + glyph_info.offset_y + run.line_y);
				let uv_pos = Vec2::new(glyph_info.uv_x, glyph_info.uv_y);
				let size = Vec2::new(glyph_info.width, glyph_info.height);
				let bounds = Aabb2::new(pos, pos + size);

				let uv_bounds = Aabb2::new(uv_pos * pixel_uv_size, (uv_pos + size) * pixel_uv_size);

				let glyph_color = match (glyph.color_opt, glyph_info.is_color_bitmap) {
					(_, true) => Color::white(),
					(Some(ct_color), _) => Color::from(ct_color.as_rgba()).to_linear(),
					(None, _) => text_color,
				};

				self.set_color(glyph_color);
				self.set_uv_rect(uv_bounds);
				self.rect(bounds);
			}
		}

		// Restore initial state
		self.set_color(text_color);
		self.set_uv_rect(None);
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



struct VertexConstructor<'r> {
	color: [f32; 4],
	clip_rect: [u16; 4],

	total_rect: Option<&'r mut Aabb2>,
}

impl VertexConstructor<'_> {
	fn new(color: impl Into<Color>, clip_rect: [u16; 4]) -> Self {
		VertexConstructor {
			color: color.into().into(),
			clip_rect,
			total_rect: None,
		}
	}
}

impl FillVertexConstructor<renderer::Vertex> for VertexConstructor<'_> {
	fn new_vertex(&mut self, vertex: FillVertex) -> renderer::Vertex {
		let pos = vertex.position().to_array();

		if let Some(rect) = self.total_rect.as_mut() {
			rect.min.x = rect.min.x.min(pos[0]);
			rect.max.x = rect.max.x.max(pos[0]);
			rect.min.y = rect.min.y.min(pos[1]);
			rect.max.y = rect.max.y.max(pos[1]);
		}

		renderer::Vertex {
			pos,
			color: self.color,
			uv: [1.0, 1.0],
			clip_rect: self.clip_rect,
		}
	}
}

impl StrokeVertexConstructor<renderer::Vertex> for VertexConstructor<'_> {
	fn new_vertex(&mut self, vertex: StrokeVertex) -> renderer::Vertex {
		let pos = vertex.position().to_array();

		if let Some(rect) = self.total_rect.as_mut() {
			rect.min.x = rect.min.x.min(pos[0]);
			rect.max.x = rect.max.x.max(pos[0]);
			rect.min.y = rect.min.y.min(pos[1]);
			rect.max.y = rect.max.y.max(pos[1]);
		}

		renderer::Vertex {
			pos,
			color: self.color,
			uv: [1.0, 1.0],
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

fn to_4u16(bounds: impl Into<Option<Aabb2>>) -> [u16; 4] {
	let Some(bounds) = bounds.into() else {
		return [0, u16::MAX, 0, u16::MAX]
	};

	let min_x = bounds.min.x.round().clamp(0.0, u16::MAX as f32) as i32 as u16;
	let max_x = bounds.max.x.round().clamp(0.0, u16::MAX as f32) as i32 as u16;
	let min_y = bounds.min.y.round().clamp(0.0, u16::MAX as f32) as i32 as u16;
	let max_y = bounds.max.y.round().clamp(0.0, u16::MAX as f32) as i32 as u16;
	[min_x, max_x, min_y, max_y]
}