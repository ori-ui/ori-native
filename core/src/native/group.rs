use crate::{
    Color, Corners, OutlineStyle, Overflow, Platform, Shadow, Sides, Unsupported,
    platform::unsupported,
};

/// A native group widget.
///
/// A group is a widget with multiple children, a background, border and a shadow.
pub trait NativeGroup<P>
where
    P: Platform,
{
    /// Build the widget.
    fn build(platform: &mut P) -> Self;

    /// Teardown the widget.
    fn teardown(self, platform: &mut P);

    /// Get a reference to the widget.
    fn widget_ref(&self) -> P::WidgetRef;

    /// Insert a `child` at `index`.
    fn insert_child(&mut self, platform: &mut P, index: usize, child: P::WidgetRef);

    /// Remove the child at `index`.
    fn remove_child(&mut self, platform: &mut P, index: usize);

    /// Replace the child at `index`.
    fn replace_child(&mut self, platform: &mut P, index: usize, child: P::WidgetRef);

    /// Swap the order of children at `index_a` and `index_b`.
    fn swap_children(&mut self, platform: &mut P, index_a: usize, index_b: usize);

    /// Set the layout rect of the child at `index`.
    fn set_child_layout(
        &mut self,
        platform: &mut P,
        index: usize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    );

    /// Set the fill `color` of the background.
    fn set_background_color(&mut self, platform: &mut P, color: Color);

    /// Set the stroke `color` of the border.
    fn set_border_color(&mut self, platform: &mut P, color: Color);

    /// Set the sidewise `widths` of the border.
    fn set_border_width(&mut self, platform: &mut P, widths: Sides<f32>);

    /// Set the outline.
    fn set_outline(&mut self, platform: &mut P, outline: OutlineStyle);

    /// Set the radii of each corner.
    fn set_corners(&mut self, platform: &mut P, corners: Corners<f32>);

    /// Set the `overflow` mode.
    fn set_overflow(&mut self, platform: &mut P, overflow: Overflow);

    /// Set the `shadow` drawn behind the background.
    fn set_shadow(&mut self, platform: &mut P, shadow: Shadow);

    /* platform specific */

    /// Set whether to use a hardware layer on `android`.
    fn set_hardware_layer(&mut self, platform: &mut P, enabled: bool) {
        let _ = platform;
        let _ = enabled;
    }
}

impl<P> NativeGroup<P> for Unsupported
where
    P: Platform,
{
    fn build(_platform: &mut P) -> Self {
        unsupported!("group widget")
    }

    fn teardown(self, _platform: &mut P) {
        unreachable!()
    }

    fn widget_ref(&self) -> P::WidgetRef {
        unreachable!()
    }

    fn insert_child(&mut self, _platform: &mut P, _index: usize, _child: P::WidgetRef) {
        unreachable!()
    }

    fn remove_child(&mut self, _platform: &mut P, _index: usize) {
        unreachable!()
    }

    fn replace_child(&mut self, _platform: &mut P, _index: usize, _child: P::WidgetRef) {
        unreachable!()
    }

    fn swap_children(&mut self, _platform: &mut P, _index_a: usize, _index_b: usize) {
        unreachable!()
    }

    fn set_child_layout(
        &mut self,
        _platform: &mut P,
        _index: usize,
        _x: f32,
        _y: f32,
        _width: f32,
        _height: f32,
    ) {
        unreachable!()
    }

    fn set_background_color(&mut self, _platform: &mut P, _color: Color) {
        unreachable!()
    }

    fn set_border_color(&mut self, _platform: &mut P, _color: Color) {
        unreachable!()
    }

    fn set_border_width(&mut self, _platform: &mut P, _width: Sides<f32>) {
        unreachable!()
    }

    fn set_outline(&mut self, _platform: &mut P, _outline: OutlineStyle) {
        unreachable!()
    }

    fn set_corners(&mut self, _platform: &mut P, _radii: Corners<f32>) {
        unreachable!()
    }

    fn set_overflow(&mut self, _platform: &mut P, _overflow: Overflow) {
        unreachable!()
    }

    fn set_shadow(&mut self, _platform: &mut P, _shadow: Shadow) {
        unreachable!()
    }
}
