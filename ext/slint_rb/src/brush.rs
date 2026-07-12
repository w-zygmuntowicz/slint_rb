use std::fmt;
use magnus::prelude::*;
use magnus::{function, method, typed_data, scan_args};

use crate::errors::{RbResult, SlintError};

#[magnus::wrap(class = "Slint::Color")]
#[derive(Default, Debug, PartialEq, PartialOrd)]
pub struct Color {
    color: slint_interpreter::Color
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
            (None, Some(red), Some(green), Some(blue), None) => Ok(Self::from_rgb_u8(red, green, blue)),
            (None, Some(red), Some(green), Some(blue), Some(alpha)) => Ok(Self::from_argb_u8(alpha, red, green, blue)),
            (Some(hex), None, None, None, None) => {
                hex
                    .parse::<css_color_parser2::Color>()
                    .map(|c| Self::from_argb_u8((c.a * 255.) as u8, c.r, c.g, c.b) )
                    .map_err(|err| SlintError::new_err(err.to_string()))
            },
            (None, ..) => Err(SlintError::new_err("Invalid keyword arguments. Expected red, green, and blue (with optional alpha).".to_string())),
            _ => Err(SlintError::new_err("Provide either a hex string or RGB(A) keywords, not both.".to_string()))
        }
    }

    fn from_rgb_u8(red: u8, green: u8, blue: u8) -> Self {
        Self { color: slint_interpreter::Color::from_rgb_u8(red, green, blue) }
    }

    fn from_argb_u8(alpha: u8, red: u8, green: u8, blue: u8) -> Self {
        Self { color: slint_interpreter::Color::from_argb_u8(alpha, red, green, blue) }
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

    pub fn transparentize(&self, factor: f32) -> Color {
        Color { color: self.color.transparentize(factor) }
    }

    pub fn brighter(&self, factor: f32) -> Color {
        Color { color: self.color.brighter(factor) }
    }

    pub fn darker(&self, factor: f32) -> Color {
        Color { color: self.color.darker(factor) }
    }

    pub fn mix(&self, other: &Color, factor: f32) -> Color {
        Color { color: self.color.mix(&other.color, factor) }
    }

    pub fn with_alpha(&self, alpha: f32) -> Color {
        Color { color: self.color.with_alpha(alpha) }
    }
}

pub fn init(ruby: &magnus::Ruby, slint_module: &magnus::RModule) -> Result<(), magnus::Error> {
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

    Ok(())
}
