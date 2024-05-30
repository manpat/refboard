use crate::prelude::*;

use cosmic_text as ct;


pub const TEXT_ATLAS_SIZE: u32 = 2048;


pub struct GlyphUpdate {
	pub image: ct::SwashImage,
	pub dst_pos: Vec2i,
}

impl std::fmt::Debug for GlyphUpdate {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "GlyphUpdate({:?})", self.dst_pos)
	}
}

#[derive(Debug)]
pub struct GlyphInfo {
	pub width: f32,
	pub height: f32,

	pub offset_x: f32,
	pub offset_y: f32,

	pub uv_x: f32,
	pub uv_y: f32,

	pub subpixel_alpha: bool,
	pub is_color_bitmap: bool,
}


#[derive(Debug)]
pub struct TextState {
	pub font_system: ct::FontSystem,
	pub swash_cache: ct::SwashCache,

	pub glyph_updates: Vec<GlyphUpdate>,
	pub glyph_info: HashMap<ct::CacheKey, GlyphInfo>,

	cursor_x: u32,
	cursor_y: u32,
	row_height: u32,
}


impl TextState {
	pub fn new() -> Self {
		TextState {
			font_system: ct::FontSystem::new(),
			swash_cache: ct::SwashCache::new(),

			glyph_updates: Vec::new(),
			glyph_info: HashMap::new(),

			cursor_x: 0,
			cursor_y: 0,
			row_height: 0,
		}
	}

	pub fn request_glyph(&mut self, key: &ct::CacheKey) -> &GlyphInfo {
		self.glyph_info.entry(*key)
			.or_insert_with(|| {
				// TODO(pat.m): we might need to use swash directly if we want subpixel raster
				let image = self.swash_cache.get_image_uncached(&mut self.font_system, *key).unwrap();
				let ct::Placement {width, height, left, top} = image.placement;

				if self.cursor_x + width > TEXT_ATLAS_SIZE {
					self.cursor_y += self.row_height;
					self.cursor_x = 0;
					self.row_height = 0;
				}

				self.row_height = self.row_height.max(height + 1);

				let info = GlyphInfo {
					width: width as f32,
					height: height as f32,

					offset_x: left as f32,
					offset_y: -top as f32,

					uv_x: self.cursor_x as f32,
					uv_y: self.cursor_y as f32,

					subpixel_alpha: image.content == ct::SwashContent::SubpixelMask,
					is_color_bitmap: image.content == ct::SwashContent::Color,
				};

				self.glyph_updates.push(GlyphUpdate {
					image,
					dst_pos: Vec2i::new(self.cursor_x as i32, self.cursor_y as i32),
				});

				self.cursor_x += width + 1;

				info
			})
	}
}