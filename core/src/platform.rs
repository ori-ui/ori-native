use ori::Proxied;

use crate::native::{
    NativeGroup, NativeImage, NativeMeasure, NativePainter, NativePopup, NativePressable,
    NativeScroll, NativeText, NativeTextInput, NativeTransform, NativeWindow,
};

/// A native platform, e.g. windows or gtk4.
pub trait Platform: Proxied + Sized + 'static {
    /// The base widget of this platform.
    type WidgetRef: Clone;

    /// The native group widget of this platform.
    type Group: NativeGroup<Self>;

    /// The native image widget of this platform.
    type Image: NativeImage<Self>;

    /// The native pressable widget of this platform.
    type Pressable: NativePressable<Self>;

    /// The native scroll widget of this platform.
    type Scroll: NativeScroll<Self>;

    /// The native text widget of this platform.
    type Text: NativeText<Self>;

    /// The native text input widget of this platform.
    type TextInput: NativeTextInput<Self>;

    /// The native transform widget of this platform.
    type Transform: NativeTransform<Self>;

    /// The native measure widget of this platform.
    type Measure: NativeMeasure<Self>;

    /// The native painter widget of this platform.
    type Painter: NativePainter<Self>;

    /// The native popup widget of this platform.
    type Popup: NativePopup<Self>;

    /// The native window widget of this platform.
    type Window: NativeWindow<Self>;

    /// Quit the application.
    fn quit(&mut self);
}

/// A widget that is not supported on a given platform.
pub struct Unsupported;

macro_rules! unsupported {
    ($($arg:tt)+) => {
        ::std::panic!("feature not supported: {}", ::std::format_args!($($arg)+))
    };
}

pub(crate) use unsupported;
