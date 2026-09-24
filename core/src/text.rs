use std::{borrow::Cow, ops::Range};

use crate::Color;

/// Description of a font.
#[derive(Clone, Debug, PartialEq)]
pub struct Font {
    /// Font size in pixels.
    pub size: f32,

    /// Font family.
    ///
    /// If `None` system default is used.
    pub family: Option<Cow<'static, str>>,

    /// Font [`Weight`].
    pub weight: Weight,

    /// Font [`Stretch`].
    pub stretch: Stretch,

    /// Whether the font is _italic_.
    pub italic: bool,

    /// Whether the font is striketrough.
    pub striketrough: bool,

    /// Color of the font.
    pub color: Color,
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

/// [`Font`] weight.
///
/// This defines the *boldness* of a [`Font`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Weight(pub u16);

impl Weight {
    /// Thin font [`Weight`].
    pub const THIN: Self = Self(100);

    /// Extra-light font [`Weight`].
    pub const EXTRA_LIGHT: Self = Self(200);

    /// Light font [`Weight`].
    pub const LIGHT: Self = Self(300);

    /// Normal font [`Weight`].
    pub const NORMAL: Self = Self(400);

    /// Medium font [`Weight`].
    pub const MEDIUM: Self = Self(500);

    /// Semi-bold font [`Weight`].
    pub const SEMI_BOLD: Self = Self(600);

    /// Bold font [`Weight`].
    pub const BOLD: Self = Self(700);

    /// Extra-bold font [`Weight`].
    pub const EXTRA_BOLD: Self = Self(800);

    /// Heavy font [`Weight`].
    pub const HEAVY: Self = Self(900);
}

/// [`Font`] stretch.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stretch {
    /// Ultra-condensed font [`Stretch`].
    UltraCondensed,

    /// Extra-condensed font [`Stretch`].
    ExtraCondensed,

    /// Condensed font [`Stretch`].
    Condensed,

    /// Semi-condensed font [`Stretch`].
    SemiCondensed,

    /// Normal font [`Stretch`].
    Normal,

    /// Semi-expanded font [`Stretch`].
    SemiExpanded,

    /// Expanded font [`Stretch`].
    Expanded,

    /// Extra-expanded font [`Stretch`].
    ExtraExpanded,

    /// Ultra-expanded font [`Stretch`].
    UltraExpanded,
}

/// Wrap mode of text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextWrap {
    /// Text is wrapped between words.
    Word,

    /// Text is wrapped between characters.
    Char,

    /// Text is not wrapped.
    None,
}

/// Alignment mode of text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextAlign {
    /// Text is aligned at the start.
    Start,

    /// Text is aligned at the center.
    Center,

    /// Text is aligned at the end.
    End,

    /// Text is justified.
    Justify,
}

/// A [`Font`] associated with a span of text.
#[derive(Clone, Debug, PartialEq)]
pub struct TextSpan {
    /// The font of the span.
    pub font: Font,

    /// The range of the span in bytes.
    pub range: Range<usize>,
}
