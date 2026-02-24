use std::{borrow::Cow, ops::Range};

use crate::Color;

#[derive(Clone, Debug, PartialEq)]
pub struct Font {
    pub size:         f32,
    pub family:       Option<Cow<'static, str>>,
    pub weight:       Weight,
    pub stretch:      Stretch,
    pub italic:       bool,
    pub striketrough: bool,
    pub color:        Color,
}

impl Default for Font {
    fn default() -> Self {
        Self {
            size:         14.0,
            family:       None,
            weight:       Weight::NORMAL,
            stretch:      Stretch::Normal,
            italic:       false,
            striketrough: false,
            color:        Color::BLACK,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Weight(pub u16);

impl Weight {
    pub const THIN: Self = Self(100);
    pub const EXTRA_LIGHT: Self = Self(200);
    pub const LIGHT: Self = Self(300);
    pub const NORMAL: Self = Self(400);
    pub const MEDIUM: Self = Self(500);
    pub const SEMI_BOLD: Self = Self(600);
    pub const BOLD: Self = Self(700);
    pub const EXTRA_BOLD: Self = Self(800);
    pub const HEAVY: Self = Self(900);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stretch {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Wrap {
    Word,
    Char,
    None,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextSpan {
    pub font:  Font,
    pub range: Range<usize>,
}
