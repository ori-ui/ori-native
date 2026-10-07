use std::time::Duration;

use keyboard_types::{Key, Modifiers};

use crate::{NavigationBar, Platform, StatusBar, Unsupported, platform::unsupported};

/// A native window.
pub trait NativeWindow<P>
where
    P: Platform,
{
    /// Build a window.
    fn build(platform: &mut P, contents: P::WidgetRef) -> Self;

    /// Teardown the window.
    fn teardown(self, platform: &mut P);

    /// Replace contents.
    fn replace_contents(&mut self, platform: &mut P, contents: P::WidgetRef);

    /// Get the current size of the window.
    fn get_size(&self, platform: &mut P) -> (f32, f32);

    /// Get the current scale factor of the window.
    fn get_scale(&self, platform: &mut P) -> f32;

    /// Get the preferred size of the window.
    ///
    /// Imagine a phone app where the window size is fixed.
    fn get_preferred_size(&self, platform: &mut P) -> (Option<f32>, Option<f32>);

    /// Check if the window is decorated.
    fn is_decorated(&self, platform: &mut P) -> bool;

    /// Set the `on_frame` callback.
    fn set_on_animation_frame(&mut self, platform: &mut P, on_frame: impl Fn(Duration) + 'static);

    /// Set the `on_resize` callback.
    fn set_on_resize(&mut self, platform: &mut P, on_resize: impl Fn() + 'static);

    /// Set the `on_close_requested` callback.
    fn set_on_close_requested(&mut self, platform: &mut P, on_close_requested: impl Fn() + 'static);

    /// Set the `on_key` callback.
    fn set_on_key(
        &mut self,
        platform: &mut P,
        on_key: impl Fn(Key, Modifiers, bool) -> bool + 'static,
    );

    /// Start requesting animation frames.
    fn start_animating(&mut self, platform: &mut P);

    /// Stop requesting animation frames.
    fn stop_animating(&mut self, platform: &mut P);

    /// Set the layout rectangle for the contents.
    fn set_content_layout(&mut self, platform: &mut P, x: f32, y: f32, width: f32, height: f32);

    /// Set the `title`.
    fn set_title(&mut self, platform: &mut P, title: String);

    /// Set the minimum size.
    fn set_min_size(&mut self, platform: &mut P, width: f32, height: f32);

    /// Set the size.
    fn set_size(&mut self, platform: &mut P, width: f32, height: f32);

    /// Set whether the window should be decorated.
    fn set_decorated(&mut self, platform: &mut P, decorated: bool);

    /// Set whether the window is resizable.
    fn set_resizable(&mut self, platform: &mut P, resizable: bool);

    /// Set the status bar configuration.
    fn set_status_bar(&mut self, platform: &mut P, bar: StatusBar);

    /// Set the navigation bar configuration.
    fn set_navigation_bar(&mut self, platform: &mut P, bar: NavigationBar);
}

impl<P> NativeWindow<P> for Unsupported
where
    P: Platform,
{
    fn build(_platform: &mut P, _contents: P::WidgetRef) -> Self {
        unsupported!("window view")
    }

    fn teardown(self, _platform: &mut P) {
        unreachable!()
    }

    fn replace_contents(&mut self, _platform: &mut P, _contents: P::WidgetRef) {
        unreachable!()
    }

    fn get_size(&self, _platform: &mut P) -> (f32, f32) {
        unreachable!()
    }

    fn get_scale(&self, _platform: &mut P) -> f32 {
        unreachable!()
    }

    fn get_preferred_size(&self, _platform: &mut P) -> (Option<f32>, Option<f32>) {
        unreachable!()
    }

    fn is_decorated(&self, _platform: &mut P) -> bool {
        unreachable!()
    }

    fn set_on_animation_frame(
        &mut self,
        _platform: &mut P,
        _on_frame: impl Fn(Duration) + 'static,
    ) {
        unreachable!()
    }

    fn set_on_resize(&mut self, _platform: &mut P, _on_resize: impl Fn() + 'static) {
        unreachable!()
    }

    fn set_on_close_requested(
        &mut self,
        _platform: &mut P,
        _on_close_requested: impl Fn() + 'static,
    ) {
        unreachable!()
    }

    fn set_on_key(
        &mut self,
        _platform: &mut P,
        _on_key: impl Fn(Key, Modifiers, bool) -> bool + 'static,
    ) {
        unreachable!()
    }

    fn start_animating(&mut self, _platform: &mut P) {
        unreachable!()
    }

    fn stop_animating(&mut self, _platform: &mut P) {
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

    fn set_title(&mut self, _platform: &mut P, _title: String) {
        unreachable!()
    }

    fn set_min_size(&mut self, _platform: &mut P, _width: f32, _height: f32) {
        unreachable!()
    }

    fn set_size(&mut self, _platform: &mut P, _width: f32, _height: f32) {
        unreachable!()
    }

    fn set_decorated(&mut self, _platform: &mut P, _decorated: bool) {
        unreachable!()
    }

    fn set_resizable(&mut self, _platform: &mut P, _resizable: bool) {
        unreachable!()
    }

    fn set_status_bar(&mut self, _platform: &mut P, _bar: StatusBar) {
        unreachable!()
    }

    fn set_navigation_bar(&mut self, _platform: &mut P, _bar: NavigationBar) {
        unreachable!()
    }
}
