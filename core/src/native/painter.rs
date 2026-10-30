use crate::{Canvas, Platform, Unsupported};

/// A native painter widget.
pub trait NativePainter<P>
where
    P: Platform,
{
    /// Build a new painter widget.
    fn build(platform: &mut P) -> Self;

    /// Teardown the widget.
    fn teardown(self, platform: &mut P);

    /// Get a reference to the widget.
    fn widget_ref(&self) -> P::WidgetRef;

    /// Set the canvas to draw.
    fn set_canvas(&mut self, platform: &mut P, canvas: Canvas);
}

impl<P> NativePainter<P> for Unsupported
where
    P: Platform,
{
    fn build(_platform: &mut P) -> Self {
        unreachable!()
    }

    fn teardown(self, _platform: &mut P) {
        unreachable!()
    }

    fn widget_ref(&self) -> P::WidgetRef {
        unreachable!()
    }

    fn set_canvas(&mut self, _platform: &mut P, _canvas: Canvas) {
        unreachable!()
    }
}
