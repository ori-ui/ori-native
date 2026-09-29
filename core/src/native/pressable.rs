use keyboard_types::{Key, Modifiers};

use crate::{Platform, Unsupported, event::PressableEvent, platform::unsupported};

/// A native widget that receives pointer input and focus.
pub trait NativePressable<P>
where
    P: Platform,
{
    /// Build a pressable.
    fn build(
        platform: &mut P,
        contents: P::WidgetRef,
        on_event: impl Fn(PressableEvent) + 'static,
    ) -> Self;

    /// Teardown the widget.
    fn teardown(self, platform: &mut P);

    /// Get a reference to the widget.
    fn widget_ref(&self) -> P::WidgetRef;

    /// Replace the contents;
    fn replace_contents(&mut self, platform: &mut P, contents: P::WidgetRef);

    /// Set the size of the contents.
    fn set_content_size(&mut self, platform: &mut P, width: f32, height: f32);

    /// Set whether the widget is scrollable.
    fn set_scrollable(&mut self, platform: &mut P, scrollable: bool);

    /// Set whether the widget is focusable.
    fn set_focusable(&mut self, platform: &mut P, focusable: bool);

    /// Set the `on_key` callback.
    fn set_on_key(
        &mut self,
        platform: &mut P,
        on_key: impl Fn(Key, Modifiers, bool) -> bool + 'static,
    );
}

impl<P> NativePressable<P> for Unsupported
where
    P: Platform,
{
    fn build(
        _platform: &mut P,
        _contents: P::WidgetRef,
        _on_event: impl Fn(PressableEvent) + 'static,
    ) -> Self {
        unsupported!("pressable view")
    }

    fn teardown(self, _platform: &mut P) {
        unreachable!()
    }

    fn widget_ref(&self) -> P::WidgetRef {
        unreachable!()
    }

    fn replace_contents(&mut self, _platform: &mut P, _contents: P::WidgetRef) {
        unreachable!()
    }

    fn set_content_size(&mut self, _platform: &mut P, _width: f32, _height: f32) {
        unreachable!()
    }

    fn set_scrollable(&mut self, _platform: &mut P, _scrollable: bool) {
        unreachable!()
    }

    fn set_focusable(&mut self, _platform: &mut P, _focusable: bool) {
        unreachable!()
    }

    fn set_on_key(
        &mut self,
        _platform: &mut P,
        _on_key: impl Fn(Key, Modifiers, bool) -> bool + 'static,
    ) {
        unreachable!()
    }
}
