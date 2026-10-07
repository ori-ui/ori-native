use crate::{Platform, PopupPosition, Unsupported, platform::unsupported};

/// A native widget that shows a popup relative to an anchor.
pub trait NativePopup<P>
where
    P: Platform,
{
    /// Build the widget with the anchor.
    fn build(platform: &mut P, anchor: P::WidgetRef, on_dismiss: impl Fn() + 'static) -> Self;

    /// Teardown the widget.
    fn teardown(self, platform: &mut P);

    /// Get a reference to the anchor widget.
    fn widget_ref(&self) -> P::WidgetRef;

    /// Replace the anchor widget.
    fn replace_anchor(&mut self, platform: &mut P, anchor: P::WidgetRef);

    /// Get the scale factor of the popup surface.
    fn get_scale(&mut self, platform: &mut P) -> f32;

    /// Open the popup with given `contents`.
    fn open(&mut self, platform: &mut P, contents: P::WidgetRef);

    /// Close the popup.
    fn close(&mut self, platform: &mut P);

    /// Set the positioning scheme of the popup.
    fn set_position(&mut self, platform: &mut P, position: PopupPosition);

    /// Set whether the popup is modal.
    fn set_modal(&mut self, platform: &mut P, is_modal: bool);

    /// Set the size of the anchor widget.
    fn set_anchor_size(&mut self, platform: &mut P, width: f32, height: f32);

    /// Set the size of the popup.
    fn set_popup_size(&mut self, platform: &mut P, width: f32, height: f32);

    /// Set the layout of the popup contents.
    fn set_content_layout(&mut self, platform: &mut P, x: f32, y: f32, width: f32, height: f32);
}

impl<P> NativePopup<P> for Unsupported
where
    P: Platform,
{
    fn build(_platform: &mut P, _anchor: P::WidgetRef, _on_dismiss: impl Fn() + 'static) -> Self {
        unsupported!("popup view")
    }

    fn teardown(self, _platform: &mut P) {
        unreachable!()
    }

    fn widget_ref(&self) -> P::WidgetRef {
        unreachable!()
    }

    fn replace_anchor(&mut self, _platform: &mut P, _anchor: P::WidgetRef) {
        unreachable!()
    }

    fn get_scale(&mut self, _platform: &mut P) -> f32 {
        unreachable!()
    }

    fn open(&mut self, _platform: &mut P, _contents: P::WidgetRef) {
        unreachable!()
    }

    fn close(&mut self, _platform: &mut P) {
        unreachable!()
    }

    fn set_position(&mut self, _platform: &mut P, _position: PopupPosition) {
        unreachable!()
    }

    fn set_modal(&mut self, _platform: &mut P, _is_modal: bool) {
        unreachable!()
    }

    fn set_popup_size(&mut self, _platform: &mut P, _width: f32, _height: f32) {
        unreachable!()
    }

    fn set_anchor_size(&mut self, _platform: &mut P, _width: f32, _height: f32) {
        unreachable!()
    }

    fn set_content_layout(
        &mut self,
        _platform: &mut P,
        _x: f32,
        _y: f32,
        _width: f32,
        _height: f32,
    ) {
        unreachable!()
    }
}
