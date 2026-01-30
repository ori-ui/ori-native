#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Percent(pub f32);

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AutoLength {
    Length(f32),
    Percent(f32),
    Auto,
}

impl From<f32> for AutoLength {
    fn from(length: f32) -> Self {
        AutoLength::Length(length)
    }
}

impl From<Percent> for AutoLength {
    fn from(Percent(percent): Percent) -> Self {
        AutoLength::Percent(percent)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Length(f32),
    Percent(f32),
}

impl From<f32> for Length {
    fn from(length: f32) -> Self {
        Length::Length(length)
    }
}

impl From<Percent> for Length {
    fn from(Percent(percent): Percent) -> Self {
        Length::Percent(percent)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Align {
    Start,
    Center,
    End,
    Baseline,
    Stretch,
    FlexStart,
    FlexEnd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Justify {
    Start,
    Center,
    End,
    Stretch,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
    FlexStart,
    FlexEnd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Position {
    Relative,
    Absolute,
}

pub trait Layout: Sized {
    fn style_mut(&mut self) -> &mut taffy::Style;

    fn position(mut self, position: Position) -> Self {
        self.style_mut().position = position.into_taffy();
        self
    }

    fn offset(mut self, x: impl Into<AutoLength>, y: impl Into<AutoLength>) -> Self {
        self.style_mut().inset.left = x.into().into_taffy_length_auto();
        self.style_mut().inset.top = y.into().into_taffy_length_auto();
        self
    }

    fn size(self, width: impl Into<AutoLength>, height: impl Into<AutoLength>) -> Self {
        self.width(width).height(height)
    }

    fn width(mut self, width: impl Into<AutoLength>) -> Self {
        self.style_mut().size.width = width.into().into_taffy_dimension();
        self
    }

    fn height(mut self, height: impl Into<AutoLength>) -> Self {
        self.style_mut().size.height = height.into().into_taffy_dimension();
        self
    }

    fn min_size(self, min_width: impl Into<AutoLength>, min_height: impl Into<AutoLength>) -> Self {
        self.min_width(min_width).min_height(min_height)
    }

    fn min_width(mut self, min_width: impl Into<AutoLength>) -> Self {
        self.style_mut().min_size.width = min_width.into().into_taffy_dimension();
        self
    }

    fn min_height(mut self, min_height: impl Into<AutoLength>) -> Self {
        self.style_mut().min_size.height = min_height.into().into_taffy_dimension();
        self
    }

    fn max_size(self, max_width: impl Into<AutoLength>, max_height: impl Into<AutoLength>) -> Self {
        self.max_width(max_width).max_height(max_height)
    }

    fn max_width(mut self, max_width: impl Into<AutoLength>) -> Self {
        self.style_mut().max_size.width = max_width.into().into_taffy_dimension();
        self
    }

    fn max_height(mut self, max_height: impl Into<AutoLength>) -> Self {
        self.style_mut().max_size.height = max_height.into().into_taffy_dimension();
        self
    }

    fn margin(self, width: impl Into<AutoLength>) -> Self {
        let width = width.into();
        self.margin_all(width, width, width, width)
    }

    fn margin_top(mut self, width: impl Into<AutoLength>) -> Self {
        self.style_mut().margin.top = width.into().into_taffy_length_auto();
        self
    }

    fn margin_right(mut self, width: impl Into<AutoLength>) -> Self {
        self.style_mut().margin.right = width.into().into_taffy_length_auto();
        self
    }

    fn margin_bottom(mut self, width: impl Into<AutoLength>) -> Self {
        self.style_mut().margin.bottom = width.into().into_taffy_length_auto();
        self
    }

    fn margin_left(mut self, width: impl Into<AutoLength>) -> Self {
        self.style_mut().margin.left = width.into().into_taffy_length_auto();
        self
    }

    fn margin_all(
        self,
        top: impl Into<AutoLength>,
        right: impl Into<AutoLength>,
        bottom: impl Into<AutoLength>,
        left: impl Into<AutoLength>,
    ) -> Self {
        self.margin_top(top)
            .margin_right(right)
            .margin_bottom(bottom)
            .margin_left(left)
    }

    fn padding(self, width: impl Into<Length>) -> Self {
        let width = width.into();
        self.padding_all(width, width, width, width)
    }

    fn padding_top(mut self, width: impl Into<Length>) -> Self {
        self.style_mut().padding.top = width.into().into_taffy();
        self
    }

    fn padding_right(mut self, width: impl Into<Length>) -> Self {
        self.style_mut().padding.right = width.into().into_taffy();
        self
    }

    fn padding_bottom(mut self, width: impl Into<Length>) -> Self {
        self.style_mut().padding.bottom = width.into().into_taffy();
        self
    }

    fn padding_left(mut self, width: impl Into<Length>) -> Self {
        self.style_mut().padding.left = width.into().into_taffy();
        self
    }

    fn padding_all(
        self,
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
        left: impl Into<Length>,
    ) -> Self {
        self.padding_top(top)
            .padding_right(right)
            .padding_bottom(bottom)
            .padding_left(left)
    }

    fn flex(self, amount: f32) -> Self {
        self.flex_grow(amount).flex_shrink(amount)
    }

    fn flex_grow(mut self, amount: f32) -> Self {
        self.style_mut().flex_grow = amount;
        self
    }

    fn flex_shrink(mut self, amount: f32) -> Self {
        self.style_mut().flex_shrink = amount;
        self
    }
}

pub trait BorderLayout: Layout {
    fn border(self, width: impl Into<Length>) -> Self {
        let width = width.into();
        self.border_all(width, width, width, width)
    }

    fn border_top(mut self, width: impl Into<Length>) -> Self {
        self.style_mut().border.top = width.into().into_taffy();
        self
    }

    fn border_right(mut self, width: impl Into<Length>) -> Self {
        self.style_mut().border.right = width.into().into_taffy();
        self
    }

    fn border_bottom(mut self, width: impl Into<Length>) -> Self {
        self.style_mut().border.bottom = width.into().into_taffy();
        self
    }

    fn border_left(mut self, width: impl Into<Length>) -> Self {
        self.style_mut().border.left = width.into().into_taffy();
        self
    }

    fn border_all(
        self,
        top: impl Into<Length>,
        right: impl Into<Length>,
        bottom: impl Into<Length>,
        left: impl Into<Length>,
    ) -> Self {
        self.border_top(top)
            .border_right(right)
            .border_bottom(bottom)
            .border_left(left)
    }
}

pub trait ContainerLayout: Layout {
    fn gap(mut self, gap: impl Into<Length>) -> Self {
        self.style_mut().gap.width = gap.into().into_taffy();
        self.style_mut().gap.height = self.style_mut().gap.width;
        self
    }
}

pub trait FlexLayout: ContainerLayout {
    fn align_items(mut self, align: Align) -> Self {
        self.style_mut().align_items = Some(align.into_taffy());
        self
    }

    fn align_contents(mut self, justify: Justify) -> Self {
        self.style_mut().align_content = Some(justify.into_taffy());
        self
    }

    fn justify_contents(mut self, justify: Justify) -> Self {
        self.style_mut().justify_content = Some(justify.into_taffy());
        self
    }
}

impl AutoLength {
    fn into_taffy_dimension(self) -> taffy::Dimension {
        match self {
            AutoLength::Length(x) => taffy::Dimension::length(x),
            AutoLength::Percent(x) => taffy::Dimension::percent(x),
            AutoLength::Auto => taffy::Dimension::auto(),
        }
    }

    fn into_taffy_length_auto(self) -> taffy::LengthPercentageAuto {
        match self {
            AutoLength::Length(x) => taffy::LengthPercentageAuto::length(x),
            AutoLength::Percent(x) => taffy::LengthPercentageAuto::percent(x),
            AutoLength::Auto => taffy::LengthPercentageAuto::auto(),
        }
    }
}

impl Length {
    fn into_taffy(self) -> taffy::LengthPercentage {
        match self {
            Length::Length(x) => taffy::LengthPercentage::length(x),
            Length::Percent(x) => taffy::LengthPercentage::percent(x),
        }
    }
}

impl Position {
    fn into_taffy(self) -> taffy::Position {
        match self {
            Position::Relative => taffy::Position::Relative,
            Position::Absolute => taffy::Position::Absolute,
        }
    }
}

impl Align {
    fn into_taffy(self) -> taffy::AlignItems {
        match self {
            Align::Start => taffy::AlignItems::Start,
            Align::Center => taffy::AlignItems::Center,
            Align::End => taffy::AlignItems::End,
            Align::Baseline => taffy::AlignItems::Baseline,
            Align::Stretch => taffy::AlignItems::Stretch,
            Align::FlexStart => taffy::AlignItems::FlexStart,
            Align::FlexEnd => taffy::AlignItems::FlexEnd,
        }
    }
}

impl Justify {
    fn into_taffy(self) -> taffy::AlignContent {
        match self {
            Justify::Start => taffy::AlignContent::Start,
            Justify::Center => taffy::AlignContent::Center,
            Justify::End => taffy::AlignContent::End,
            Justify::Stretch => taffy::AlignContent::Stretch,
            Justify::SpaceBetween => taffy::AlignContent::SpaceBetween,
            Justify::SpaceEvenly => taffy::AlignContent::SpaceEvenly,
            Justify::SpaceAround => taffy::AlignContent::SpaceAround,
            Justify::FlexStart => taffy::AlignContent::FlexStart,
            Justify::FlexEnd => taffy::AlignContent::FlexEnd,
        }
    }
}
