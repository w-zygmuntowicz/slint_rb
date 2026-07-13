use std::fmt;
use magnus::prelude::*;
use magnus::{function, method, typed_data, scan_args};

use crate::errors::{RbResult, SlintError};

#[magnus::wrap(class = "Slint::Color")]
#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Color {
    color: slint_interpreter::Color
}

impl From<slint_interpreter::Color> for Color {
    fn from(color: slint_interpreter::Color) -> Self {
        Self {
            color: color
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.color.to_string())
    }
}

impl Color {
    pub fn new(maybe_input: &[magnus::Value]) -> RbResult<Self> {
        let args = scan_args::scan_args::<(), (Option<String>,), (), (), magnus::RHash, ()>(maybe_input)?;
        let (color_string,) = args.optional;
        let kwargs = scan_args::get_kwargs::<_, (), (Option<u8>, Option<u8>, Option<u8>, Option<u8>), ()>(args.keywords, &[], &["red", "green", "blue", "alpha"])?;

        let (red, green, blue, alpha) = kwargs.optional;

        match (color_string, red, green, blue, alpha) {
            (None, None, None, None, None) => Ok(Self::default()),
            (None, maybe_red, maybe_green, maybe_blue, None) => {
                Ok(Self::from_rgb_u8(
                    maybe_red.unwrap_or(0),
                    maybe_green.unwrap_or(0),
                    maybe_blue.unwrap_or(0)
                ))
            },
            (None, maybe_red, maybe_green, maybe_blue, Some(alpha)) => {
                Ok(Self::from_argb_u8(
                    alpha,
                    maybe_red.unwrap_or(0),
                    maybe_green.unwrap_or(0),
                    maybe_blue.unwrap_or(0)
                ))
            },
            (Some(hex), None, None, None, None) => {
                hex
                    .parse::<css_color_parser2::Color>()
                    .map(|c| Self::from_argb_u8((c.a * 255.) as u8, c.r, c.g, c.b) )
                    .map_err(|err| SlintError::new_err(err.to_string()))
            },
            _ => Err(SlintError::new_err("Provide either a CSS color string or RGB(A) keywords, not both.".to_string()))
        }
    }

    fn from_rgb_u8(red: u8, green: u8, blue: u8) -> Self {
        slint_interpreter::Color::from_rgb_u8(red, green, blue).into()
    }

    fn from_argb_u8(alpha: u8, red: u8, green: u8, blue: u8) -> Self {
        slint_interpreter::Color::from_argb_u8(alpha, red, green, blue).into()
    }

    pub fn red(&self) -> u8 {
        self.color.red()
    }

    pub fn green(&self) -> u8 {
        self.color.green()
    }

    pub fn blue(&self) -> u8 {
        self.color.blue()
    }

    pub fn alpha(&self) -> u8 {
        self.color.alpha()
    }

    pub fn transparentize(&self, factor: f32) -> Self {
        self.color.transparentize(factor).into()
    }

    pub fn brighter(&self, factor: f32) -> Self {
        self.color.brighter(factor).into()
    }

    pub fn darker(&self, factor: f32) -> Self {
        self.color.darker(factor).into()
    }

    pub fn mix(&self, other: &Color, factor: f32) -> Self {
        self.color.mix(&other.color, factor).into()
    }

    pub fn with_alpha(&self, alpha: f32) -> Self {
        self.color.with_alpha(alpha).into()
    }
}

#[magnus::wrap(class = "Slint::Brush")]
#[derive(Clone)]
pub struct Brush {
    brush: slint_interpreter::Brush
}

impl From<slint_interpreter::Brush> for Brush {
    fn from(brush: slint_interpreter::Brush) -> Self {
        Self {
            brush: brush
        }
    }
}

impl From<&Brush> for slint_interpreter::Brush {
    fn from(value: &Brush) -> Self {
        value.brush.clone()
    }
}

impl Brush {
    pub fn solid(color: &Color) -> Self {
        slint_interpreter::Brush::SolidColor(color.color).into()
    }

    pub fn color(&self) -> Color {
        self.brush.color().into()
    }

    pub fn is_transparent(&self) -> bool {
        self.brush.is_transparent()
    }

    pub fn is_opaque(&self) -> bool {
        self.brush.is_opaque()
    }

    pub fn brighter(&self, factor: f32) -> Self {
        self.brush.brighter(factor).into()
    }

    pub fn darker(&self, factor: f32) -> Self {
        self.brush.darker(factor).into()
    }

    pub fn transparentize(&self, amount: f32) -> Self {
        self.brush.transparentize(amount).into()
    }

    pub fn with_alpha(&self, alpha: f32) -> Self {
        self.brush.with_alpha(alpha).into()
    }
}

pub fn init(ruby: &magnus::Ruby, slint_module: &magnus::RModule) -> RbResult<()> {
    let color_class = slint_module.define_class("Color", ruby.class_object())?;
    color_class.define_singleton_method("new", function!(Color::new, -1))?;
    color_class.define_method("red", method!(Color::red, 0))?;
    color_class.define_method("green", method!(Color::green, 0))?;
    color_class.define_method("blue", method!(Color::blue, 0))?;
    color_class.define_method("alpha", method!(Color::alpha, 0))?;
    color_class.define_method("transparentize", method!(Color::transparentize, 1))?;
    color_class.define_method("brighter", method!(Color::brighter, 1))?;
    color_class.define_method("darker", method!(Color::darker, 1))?;
    color_class.define_method("mix", method!(Color::mix, 2))?;
    color_class.define_method("with_alpha", method!(Color::with_alpha, 1))?;
    color_class.define_method("to_s", method!(Color::to_string, 0))?;
    color_class.define_method("inspect", method!(<Color as typed_data::Inspect>::inspect, 0),)?;
    // <=> sort operator based on Rust PartialOrd impl
    color_class.define_method("<=>", method!(<Color as typed_data::Cmp>::cmp, 1))?;
    // defines <, <=, >, >=, and == based on <=>
    color_class.include_module(ruby.module_comparable())?;

    let brush_class = slint_module.define_class("Brush", ruby.class_object())?;
    brush_class.define_singleton_method("solid", function!(Brush::solid, 1))?;
    brush_class.define_method("color", method!(Brush::color, 0))?;
    brush_class.define_method("transparent?", method!(Brush::is_transparent, 0))?;
    brush_class.define_method("opaque?", method!(Brush::is_opaque, 0))?;
    brush_class.define_method("brighter", method!(Brush::brighter, 1))?;
    brush_class.define_method("darker", method!(Brush::darker, 1))?;
    brush_class.define_method("transparentize", method!(Brush::transparentize, 1))?;
    brush_class.define_method("with_alpha", method!(Brush::with_alpha, 1))?;

    Ok(())
}
