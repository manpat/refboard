use crate::prelude::*;
use material_colors::{
	dynamic_color::{self, DynamicScheme},
	color::Argb,
};

// https://m3.material.io/styles/color/roles
// https://docs.rs/material-colors/latest/material_colors/
// https://codelabs.developers.google.com/color-contrast-accessibility?hl=en#0

#[derive(Debug, Default, Clone)]
pub struct WidgetStyle {
	pub outline: Option<WidgetOutlineStyle>,
	pub fill: Option<WidgetColor>,
	pub rounding: Option<painter::BorderRadii>,
}

impl WidgetStyle {
	pub fn set_fill(&mut self, color: impl Into<WidgetColor>) {
		self.fill = Some(color.into());
	}

	pub fn set_outline(&mut self, outline: impl Into<WidgetOutlineStyle>) {
		self.outline = Some(outline.into());
	}
}

impl WidgetStyle {
	pub fn text_color(&self, app_style: &AppStyle) -> Color {
		app_style.resolve_color_role(self.text_color_role())
	}

	pub fn text_color_role(&self) -> WidgetColorRole {
		match self.fill {
			None => WidgetColorRole::OnSurface,
			Some(WidgetColor::Role(role)) => role.on_color(),
			Some(WidgetColor::Literal(_)) => todo!("Pick an appropriate color based on luminance"),
		}
	}

	pub fn rounding(&self, app_style: &AppStyle) -> painter::BorderRadii {
		match self.rounding {
			Some(rounding) => rounding,
			None => painter::BorderRadii::new(app_style.frame_rounding),
		}
	}
}


#[derive(Debug, Clone)]
pub struct WidgetOutlineStyle {
	pub color: WidgetColor,
	pub width: f32,
}

impl<T> From<T> for WidgetOutlineStyle
	where T: Into<WidgetColor>
{
	fn from(color: T) -> Self {
		Self {
			color: color.into(),
			width: 1.0,
		}
	}
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
	
	Secondary,
	OnSecondary,
	SecondaryContainer,
	OnSecondaryContainer,
	
	Tertiary,
	OnTertiary,
	TertiaryContainer,
	OnTertiaryContainer,
	
	Error,
	OnError,
	ErrorContainer,
	OnErrorContainer,

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

impl WidgetColorRole {
	fn on_color(&self) -> Self {
		use WidgetColorRole::*;

		match self {
			Primary => OnPrimary,
			PrimaryContainer => OnPrimaryContainer,

			Secondary => OnSecondary,
			SecondaryContainer => OnSecondaryContainer,
			
			Tertiary => OnTertiary,
			TertiaryContainer => OnTertiaryContainer,
			
			Error => OnError,
			ErrorContainer => OnErrorContainer,

			_ => OnSurface,
		}
	}
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
		let source = Argb::from_u32(0xFF_820e35);
		// let source = Argb::from_u32(0xFF_3f574f);
		// let variant = dynamic_color::Variant::Fidelity;
		let variant = dynamic_color::Variant::TonalSpot;
		// let variant = dynamic_color::Variant::Content;
		let dark_theme = true;
		let contrast = None;

		AppStyle {
			scheme: DynamicScheme::by_variant(source, &variant, dark_theme, contrast),
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

			Secondary => self.scheme.secondary(),
			OnSecondary => self.scheme.on_secondary(),
			SecondaryContainer => self.scheme.secondary_container(),
			OnSecondaryContainer => self.scheme.on_secondary_container(),

			Tertiary => self.scheme.tertiary(),
			OnTertiary => self.scheme.on_tertiary(),
			TertiaryContainer => self.scheme.tertiary_container(),
			OnTertiaryContainer => self.scheme.on_tertiary_container(),

			Error => self.scheme.error(),
			OnError => self.scheme.on_error(),
			ErrorContainer => self.scheme.error_container(),
			OnErrorContainer => self.scheme.on_error_container(),

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

