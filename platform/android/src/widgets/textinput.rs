use jni::{
    EnvUnowned, jni_sig, jni_str,
    objects::{JObject, JString},
};
use ori_native_core::{
    AvailableSpace, Font, Measurable, Newline, Size, TextAlign, TextWrap, native::NativeTextInput,
};

use crate::{
    Platform,
    application::{GlobalState, WidgetEvent},
    platform::WidgetId,
};

pub struct TextInput {
    id: WidgetId,
}

impl NativeTextInput<Platform> for TextInput {
    fn build(
        platform: &mut Platform,
        on_change: impl Fn(String) + 'static,
        on_submit: impl Fn(String) + 'static,
    ) -> Self {
        let id = platform.next_id();

        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("createTextInput"),
                jni_sig!((long)),
                &[id.into()],
            )?
            .v()
        });

        platform.add_handler(id, move |event| match event {
            WidgetEvent::Change(text) => on_change(text.clone()),
            WidgetEvent::Submit(_) => {}
            _ => unreachable!(),
        });

        platform.add_handler(id, move |event| match event {
            WidgetEvent::Submit(text) => on_submit(text.clone()),
            WidgetEvent::Change(_) => {}
            _ => unreachable!(),
        });

        Self { id }
    }

    fn teardown(self, platform: &mut Platform) {
        platform.remove_widget(self.id);
    }

    fn widget_ref(&self) -> WidgetId {
        self.id
    }

    fn set_newline(&mut self, platform: &mut Platform, newline: Newline) {
        let singleline = matches!(newline, Newline::None);

        let _ = platform.jni(|env, activity| {
            env.call_method(
                activity,
                jni_str!("textInputSetSingleLine"),
                jni_sig!((long, boolean)),
                &[self.id.into(), singleline.into()],
            )?
            .v()
        });
    }

    fn set_accept_tab(&mut self, _platform: &mut Platform, _accept_tab: bool) {}

    fn set_font(
        &mut self,
        platform: &mut Platform,
        font: Font,
        _align: TextAlign,
        _wrap: TextWrap,
    ) {
        let _ = platform.jni(|env, activity| {
            let family = match font.family {
                Some(family) => env.new_string(family)?,
                None => JString::null(),
            };

            env.call_method(
                activity,
                jni_str!("textInputSetFont"),
                jni_sig!(
                    (
                        long,
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
                    font.size.into(),
                    (&family).into(),
                    (font.weight.0 as i32).into(),
                    0i32.into(),
                    font.italic.into(),
                    font.striketrough.into(),
                    font.color.r.into(),
                    font.color.g.into(),
                    font.color.b.into(),
                    font.color.a.into(),
                ],
            )?
            .v()
        });
    }

    fn set_text(&mut self, platform: &mut Platform, text: String) {
        let _ = platform.jni(|env, activity| {
            let text = env.new_string(text)?;

            env.call_method(
                activity,
                jni_str!("textInputSetText"),
                jni_sig!((long, JString)),
                &[self.id.into(), (&text).into()],
            )?
            .v()
        });
    }

    fn set_placeholder_font(
        &mut self,
        platform: &mut Platform,
        font: Font,
        _align: TextAlign,
        _wrap: TextWrap,
    ) {
        let _ = platform.jni(|env, activity| {
            let family = match font.family {
                Some(family) => env.new_string(family)?,
                None => JString::null(),
            };

            env.call_method(
                activity,
                jni_str!("textInputSetPlaceholderFont"),
                jni_sig!(
                    (
                        long,
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
                    font.size.into(),
                    (&family).into(),
                    (font.weight.0 as i32).into(),
                    0i32.into(),
                    font.italic.into(),
                    font.striketrough.into(),
                    font.color.r.into(),
                    font.color.g.into(),
                    font.color.b.into(),
                    font.color.a.into(),
                ],
            )?
            .v()
        });
    }

    fn set_placeholder_text(&mut self, platform: &mut Platform, text: String) {
        let _ = platform.jni(|env, activity| {
            let text = env.new_string(text)?;

            env.call_method(
                activity,
                jni_str!("textInputSetPlaceholderText"),
                jni_sig!((long, JString)),
                &[self.id.into(), (&text).into()],
            )?
            .v()
        });
    }

    fn get_measureable(&mut self, _platform: &mut Platform) -> impl Measurable<Platform> {
        TextInputLayout {
            id:     self.id,
            height: None,
        }
    }
}

pub struct TextInputLayout {
    id:     WidgetId,
    height: Option<f32>,
}

impl Measurable<Platform> for TextInputLayout {
    fn measure(
        &mut self,
        platform: &mut Platform,
        _known_size: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        let height = *self.height.get_or_insert_with(|| {
            platform
                .jni(|env, activity| {
                    env.call_method(
                        activity,
                        jni_str!("textInputMeasureHeight"),
                        jni_sig!((long) -> float),
                        &[self.id.into()],
                    )?
                    .f()
                })
                .unwrap_or(0.0)
        });

        Size { width: 0.0, height }
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_ori_OriEditText_onChange<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    id: i64,
    text: JString<'local>,
) {
    let text = text.to_string();
    GlobalState::event(
        WidgetId::new(id as u64),
        WidgetEvent::Change(text),
    );
}

#[unsafe(no_mangle)]
extern "system" fn Java_ori_OriEditText_onSubmit<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    id: i64,
    text: JString<'local>,
) {
    let text = text.to_string();
    GlobalState::event(
        WidgetId::new(id as u64),
        WidgetEvent::Submit(text),
    );
}
