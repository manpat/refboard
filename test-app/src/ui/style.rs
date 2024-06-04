use crate::prelude::*;
use material_colors::{
	dynamic_color::{self, DynamicScheme},
	color::Argb,
};

// https://docs.rs/material-colors/latest/material_colors/
// https://m3.material.io/styles/color/roles
// https://codelabs.developers.google.com/color-contrast-accessibility?hl=en#0

#[derive(Debug, Default, Clone)]
pub struct WidgetStyle {
	pub outline: Option<WidgetOutlineStyle>,
	pub fill: Option<WidgetColor>,
	pub rounding: Option<painter::BorderRadii>,
}


#[derive(Debug, Clone)]
pub struct WidgetOutlineStyle {
	pub color: WidgetColor,
	pub width: f32,
}




#[derive(Debug, Clone, Copy)]
pub enum WidgetColor {
	Literal(Color),
	Role(WidgetColorRole),
}

impl WidgetColor {
	pub fn resolve(&self, app_style: &AppStyle) -> Color {
		match self {
			WidgetColor::Literal(c) => *c,
			WidgetColor::Role(role) => app_style.resolve_color_role(*role),
		}
	}
}


#[derive(Debug, Clone, Copy)]
pub enum WidgetColorRole {
	Primary,
	OnPrimary,
	PrimaryContainer,
	OnPrimaryContainer,

	Surface,
	OnSurface,

	SurfaceContainerHighest,
	SurfaceContainerHigh,
	SurfaceContainer,
	SurfaceContainerLow,
	SurfaceContainerLowest,

	Outline,
	OutlineVariant,
}


impl From<Color> for WidgetColor {
	fn from(o: Color) -> Self {
		Self::Literal(o)
	}
}

impl From<WidgetColorRole> for WidgetColor {
	fn from(o: WidgetColorRole) -> Self {
		Self::Role(o)
	}
}




pub struct AppStyle {
	pub scheme: DynamicScheme,
	pub frame_rounding: f32,
}

impl AppStyle {
	pub fn new() -> AppStyle {
		let source = Argb::from_u32(0xFFDD6688);

		AppStyle {
			scheme: DynamicScheme::by_variant(source, &dynamic_color::Variant::Content, true, None),
			frame_rounding: 4.0,
		}
	}

	pub fn resolve_color_role(&self, role: WidgetColorRole) -> Color {
		use WidgetColorRole::*;

		let Argb{alpha, red, green, blue} = match role {
			Primary => self.scheme.primary(),
			OnPrimary => self.scheme.on_primary(),
			PrimaryContainer => self.scheme.primary_container(),
			OnPrimaryContainer => self.scheme.on_primary_container(),

			Surface => self.scheme.surface(),
			OnSurface => self.scheme.on_surface(),

			SurfaceContainerHighest => self.scheme.surface_container_highest(),
			SurfaceContainerHigh => self.scheme.surface_container_high(),
			SurfaceContainer => self.scheme.surface_container(),
			SurfaceContainerLow => self.scheme.surface_container_low(),
			SurfaceContainerLowest => self.scheme.surface_container_lowest(),

			Outline => self.scheme.outline(),
			OutlineVariant => self.scheme.outline_variant(),
		};

		Color::from([red, green, blue, alpha]).to_linear()
	}
}

