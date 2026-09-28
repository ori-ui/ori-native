use ori::{AnyView, Base, Builder, BuilderMarker, views};

use crate::{
    Context, Layout, LayoutStyle, Length, Padding, Platform, SafeAreaInsets, Sides, WidgetView,
    views::Flex,
};

/// [`View`](ori::View) that ensures contents isn't overlapped by system elements.
pub fn safe_area<V>(contents: V) -> SafeArea<V> {
    SafeArea::new(contents)
}

/// [`View`](ori::View) that ensures contents isn't overlapped by system elements.
pub struct SafeArea<V> {
    contents: V,
    style:    LayoutStyle,
}

impl<V> SafeArea<V> {
    /// Create new [`SafeArea`].
    pub fn new(contents: V) -> Self {
        Self {
            contents,
            style: LayoutStyle::default(),
        }
    }
}

impl<V> Layout for SafeArea<V> {
    fn get_layout_style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.style
    }
}

impl<V> BuilderMarker for SafeArea<V> {}
impl<P, T, V> Builder<Context<P>, T> for SafeArea<V>
where
    P: Platform,
    T: 'static,
    V: WidgetView<P, T> + 'static,
{
    fn build(self) -> Box<dyn AnyView<Context<P>, T, <Context<P> as Base>::Element>> {
        Box::new(views::using_or_default(
            move |_, insets: &SafeAreaInsets| {
                let SafeAreaInsets(insets) = insets;

                let padding = Sides {
                    top:    Length::Length(insets.top),
                    right:  Length::Length(insets.right),
                    bottom: Length::Length(insets.bottom),
                    left:   Length::Length(insets.left),
                };

                let mut flex = Flex::new(self.contents);
                *flex.get_layout_style_mut() = self.style;
                *flex.get_padding_mut() = padding;

                flex
            },
        ))
    }
}
