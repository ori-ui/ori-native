use ori::{Builder, BuilderMarker, views::using_or_default};

use crate::{
    Context, LayoutStyle, Length, Platform, SafeAreaInsets, Sides, StyleLayout, StylePadding,
    WidgetView, views::Flex,
};

/// [`View`](ori::View) that ensures contents isn't overlapped by system elements.
pub fn safe_area<V>(contents: V) -> SafeArea<V> {
    SafeArea::new(contents)
}

/// [`View`](ori::View) that ensures contents isn't overlapped by system elements.
pub struct SafeArea<V> {
    contents: V,
    layout:   LayoutStyle,
}

impl<V> SafeArea<V> {
    /// Create new [`SafeArea`].
    pub fn new(contents: V) -> Self {
        Self {
            contents,
            layout: LayoutStyle::default(),
        }
    }
}

impl<V> StyleLayout for SafeArea<V> {
    fn get_layout_style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.layout
    }
}

impl<V> BuilderMarker for SafeArea<V> {}
impl<P, T, V> Builder<Context<P>, T> for SafeArea<V>
where
    P: Platform,
    V: WidgetView<P, T>,
{
    fn build(self) -> impl WidgetView<P, T> {
        using_or_default(move |_, SafeAreaInsets(insets)| {
            let padding = Sides {
                top:    Length::Length(insets.top),
                right:  Length::Length(insets.right),
                bottom: Length::Length(insets.bottom),
                left:   Length::Length(insets.left),
            };

            Flex::new(self.contents)
                .layout(self.layout)
                .padding(padding)
        })
    }
}
