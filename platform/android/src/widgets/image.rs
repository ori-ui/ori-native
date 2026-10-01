use std::{borrow::Cow, convert::Infallible};

use jni::{jni_sig, jni_str};
use ori_native_core::{AvailableSpace, Color, Measurable, Size, native::NativeImage};

use crate::{Platform, platform::WidgetId};

pub struct Image {
    id: WidgetId,
}

impl NativeImage<Platform> for Image {
    type Error = Infallible;

    fn build(platform: &mut Platform) -> Self {
        let id = platform.next_id();

        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("createImage"),
                jni_sig!((long)),
                &[id.into()],
            )?
            .v()
        });

        Self { id }
    }

    fn teardown(self, platform: &mut Platform) {
        platform.remove_widget(self.id);
    }

    fn widget_ref(&self) -> WidgetId {
        self.id
    }

    fn load_data(
        &mut self,
        platform: &mut Platform,
        data: Cow<'static, [u8]>,
    ) -> Result<impl Measurable<Platform>, Self::Error> {
        let _ = platform.jni(|env, activity| {
            let bytes = env.byte_array_from_slice(&data)?;

            if data.windows(4).take(64).any(|w| w == b"<svg") {
                env.call_method(
                    activity,
                    jni_str!("imageLoadSvg"),
                    jni_sig!((long, byte[])),
                    &[self.id.into(), (&bytes).into()],
                )?
                .v()
            } else {
                env.call_method(
                    activity,
                    jni_str!("imageLoadBitmap"),
                    jni_sig!((long, byte[])),
                    &[self.id.into(), (&bytes).into()],
                )?
                .v()
            }
        });

        Ok(ImageLayout {
            id:     self.id,
            width:  None,
            height: None,
        })
    }

    fn set_tint(&mut self, platform: &mut Platform, tint: Option<Color>) {
        let color = tint.unwrap_or(Color::TRANSPARENT);

        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("imageSetTint"),
                jni_sig!((
                    long, boolean, float, float, float, float
                )),
                &[
                    self.id.into(),
                    tint.is_some().into(),
                    color.r.into(),
                    color.g.into(),
                    color.b.into(),
                    color.a.into(),
                ],
            )?
            .f()
        });
    }
}

pub struct ImageLayout {
    id:     WidgetId,
    width:  Option<f32>,
    height: Option<f32>,
}

impl Measurable<Platform> for ImageLayout {
    fn measure(
        &mut self,
        platform: &mut Platform,
        known_size: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> (Size<f32>, Option<f32>) {
        let width = *self.width.get_or_insert_with(|| {
            platform
                .jni(|env, activity| {
                    env.call_method(
                        activity,
                        jni_str!("imageGetWidth"),
                        jni_sig!((long) -> float),
                        &[self.id.into()],
                    )?
                    .f()
                })
                .unwrap_or(0.0)
        });

        let height = *self.height.get_or_insert_with(|| {
            platform
                .jni(|env, activity| {
                    env.call_method(
                        activity,
                        jni_str!("imageGetHeight"),
                        jni_sig!((long) -> float),
                        &[self.id.into()],
                    )?
                    .f()
                })
                .unwrap_or(0.0)
        });

        let width = match available_space.width {
            AvailableSpace::Definite(available) => available.min(width),
            AvailableSpace::MinContent => 0.0,
            AvailableSpace::MaxContent => width,
        };

        let height = match available_space.height {
            AvailableSpace::Definite(available) => available.min(height),
            AvailableSpace::MinContent => 0.0,
            AvailableSpace::MaxContent => height,
        };

        let size = Size {
            width:  known_size.width.unwrap_or(width),
            height: known_size.height.unwrap_or(height),
        };

        (size, None)
    }
}
