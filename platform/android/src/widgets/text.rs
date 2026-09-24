use jni::{jni_sig, jni_str, objects::JString};
use ori_native_core::{
    AvailableSpace, Measurable, Size, TextAlign, TextSpan, TextWrap, native::NativeText,
};

use crate::{Platform, platform::WidgetId};

pub struct Text {
    id: WidgetId,
}

impl NativeText<Platform> for Text {
    fn build(platform: &mut Platform) -> Self {
        let id = platform.next_id();

        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("createText"),
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

    fn set_text(
        &mut self,
        platform: &mut Platform,
        spans: Box<[TextSpan]>,
        text: String,
        _align: TextAlign,
        wrap: TextWrap,
    ) -> impl Measurable<Platform> {
        let _ = platform.jni(|env, activity| {
            let jstring = env.new_string(&text)?;

            let wrap = match wrap {
                TextWrap::None => 3,
                TextWrap::Char => 1,
                TextWrap::Word => 2,
            };

            env.call_method(
                activity,
                jni_str!("textSetText"),
                jni_sig!((long, JString, int)),
                &[self.id.into(), (&jstring).into(), wrap.into()],
            )?
            .v()?;

            for span in spans {
                let family = match span.font.family {
                    Some(family) => env.new_string(family)?,
                    None => JString::null(),
                };

                let start = text
                    .char_indices()
                    .enumerate()
                    .find(|(_, (offset, _))| *offset == span.range.start)
                    .map(|(i, _)| i)
                    .unwrap_or(0);

                let end = text
                    .char_indices()
                    .enumerate()
                    .find(|(_, (offset, _))| *offset == span.range.end)
                    .map(|(i, _)| i)
                    .unwrap_or_else(|| text.chars().count());

                env.call_method(
                    activity,
                    jni_str!("textSetSpan"),
                    jni_sig!(
                        (
                            long,
                            int,
                            int,
                            float,
                            JString,
                            int,
                            int,
                            boolean,
                            boolean,
                            float,
                            float,
                            float,
                            float,
                        ) -> void
                    ),
                    &[
                        self.id.into(),
                        (start as i32).into(),
                        (end as i32).into(),
                        span.font.size.into(),
                        (&family).into(),
                        (span.font.weight.0 as i32).into(),
                        0i32.into(),
                        span.font.italic.into(),
                        span.font.striketrough.into(),
                        span.font.color.r.into(),
                        span.font.color.g.into(),
                        span.font.color.b.into(),
                        span.font.color.a.into(),
                    ],
                )?
                .v()?;
            }

            Ok::<_, jni::errors::Error>(())
        });

        TextLayout { id: self.id }
    }
}

pub struct TextLayout {
    id: WidgetId,
}

impl Measurable<Platform> for TextLayout {
    fn measure(
        &mut self,
        platform: &mut Platform,
        _known_size: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        let width = platform
            .jni(|env, activity| {
                env.call_method(
                    activity,
                    jni_str!("textMeasureWidth"),
                    jni_sig!((long, float) -> float),
                    &[self.id.into(), 10000.0f32.into()],
                )?
                .f()
            })
            .unwrap_or(0.0);

        let height = platform
            .jni(|env, activity| {
                env.call_method(
                    activity,
                    jni_str!("textMeasureHeight"),
                    jni_sig!((long, float) -> float),
                    &[self.id.into(), 10000.0f32.into()],
                )?
                .f()
            })
            .unwrap_or(0.0);

        Size {
            width: width + 1.0,
            height,
        }
    }
}
