use std::{cell::Cell, rc::Rc};

use glib::object::{Cast, ObjectExt};
use gtk4::prelude::{TextBufferExt, TextTagExt, TextViewExt, WidgetExt};
use ori_native_core::{
    AvailableSpace, Color, Font, Measurable, Newline, Size, Stretch, Submit, TextAlign,
    TextInputEvent, TextWrap, native::NativeTextInput,
};

use crate::{Platform, platform::StyleNode};

pub struct TextInput {
    overlay:     gtk4::Overlay,
    view:        gtk4::TextView,
    placeholder: gtk4::TextView,

    view_style: StyleNode,

    font:    Font,
    newline: Rc<Cell<Newline>>,
    submit:  Rc<Cell<Submit>>,
}

impl NativeTextInput<Platform> for TextInput {
    fn build(platform: &mut Platform, on_event: impl Fn(TextInputEvent) + 'static) -> Self {
        let overlay = gtk4::Overlay::new();
        let view = gtk4::TextView::new();
        let placeholder = gtk4::TextView::new();
        placeholder.set_sensitive(false);
        placeholder.set_visible(false);

        overlay.set_child(Some(&view));
        overlay.add_overlay(&placeholder);

        let view_style = platform.add_style("");
        view.add_css_class(&view_style.class());

        let newline = Rc::new(Cell::new(Newline::Enter));
        let submit = Rc::new(Cell::new(Submit::Nothing));
        let on_event = Rc::new(on_event);

        let controller = gtk4::EventControllerFocus::new();
        controller.connect_enter({
            let placeholder = placeholder.clone();
            let on_event = on_event.clone();

            move |_| {
                placeholder.set_visible(false);
                on_event(TextInputEvent::Focused(true));
            }
        });

        controller.connect_leave({
            let view = view.downgrade();
            let placeholder = placeholder.clone();
            let on_event = on_event.clone();

            move |_| {
                if let Some(view) = view.upgrade()
                    && view.buffer().start_iter() == view.buffer().end_iter()
                {
                    placeholder.set_visible(true);
                }

                on_event(TextInputEvent::Focused(false));
            }
        });

        view.add_controller(controller);

        view.buffer().connect_text_notify({
            let on_event = on_event.clone();

            move |buffer| {
                let text = buffer.text(
                    &buffer.start_iter(),
                    &buffer.end_iter(),
                    true,
                );

                buffer.tag_table().foreach(|tag| {
                    buffer.apply_tag(
                        tag,
                        &buffer.start_iter(),
                        &buffer.end_iter(),
                    );
                });

                on_event(TextInputEvent::Changed(text.into()));
            }
        });

        let controller = gtk4::EventControllerKey::new();

        controller.connect_key_pressed({
            let view = view.downgrade();
            let newline = newline.clone();
            let submit = submit.clone();

            move |_, key, _, state| {
                let shift = state.contains(gdk4::ModifierType::SHIFT_MASK);

                let can_submit = match newline.get() {
                    Newline::Never => true,
                    Newline::ShiftEnter if !shift => true,
                    _ => false,
                };

                if key == gdk4::Key::Return && can_submit {
                    on_event(TextInputEvent::Submitted);

                    match submit.get() {
                        Submit::Nothing => {}
                        Submit::Blur => {
                            if let Some(view) = view.upgrade() {
                                view.emit_move_focus(gtk4::DirectionType::TabForward);
                            }
                        }
                    }

                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            }
        });

        view.add_controller(controller);

        Self {
            overlay,
            view,
            placeholder,

            view_style,

            font: Default::default(),
            newline,
            submit,
        }
    }

    fn teardown(self, platform: &mut Platform) {
        platform.remove_style(self.view_style);
    }

    fn widget_ref(&self) -> gtk4::Widget {
        self.overlay.clone().upcast()
    }

    fn set_newline(&mut self, _platform: &mut Platform, newline: Newline) {
        self.newline.set(newline);
    }

    fn set_submit(&mut self, _platform: &mut Platform, submit: Submit) {
        self.submit.set(submit);
    }

    fn set_accept_tab(&mut self, _platform: &mut Platform, accept_tab: bool) {
        self.view.set_accepts_tab(accept_tab);
    }

    fn set_font(&mut self, platform: &mut Platform, font: Font, align: TextAlign, wrap: TextWrap) {
        let justification = match align {
            TextAlign::Start => gtk4::Justification::Left,
            TextAlign::Center => gtk4::Justification::Center,
            TextAlign::End => gtk4::Justification::Right,
            TextAlign::Justify => gtk4::Justification::Fill,
        };

        let wrap_mode = match wrap {
            TextWrap::Word => gtk4::WrapMode::Word,
            TextWrap::Char => gtk4::WrapMode::Char,
            TextWrap::None => gtk4::WrapMode::None,
        };

        self.view.set_justification(justification);
        self.placeholder.set_justification(justification);
        self.view.set_wrap_mode(wrap_mode);
        self.placeholder.set_wrap_mode(wrap_mode);

        platform.set_style(self.view_style, &font_style(&font));

        let buffer = self.placeholder.buffer();
        let tag_table = buffer.tag_table();
        let tag = super::text::font_tag(&font);

        tag_table.foreach(|tag| tag_table.remove(tag));
        tag_table.add(&tag);

        buffer.apply_tag(
            &tag,
            &buffer.start_iter(),
            &buffer.end_iter(),
        );

        self.font = font;
    }

    fn set_placeholder_color(&mut self, _platform: &mut Platform, color: Color) {
        let buffer = self.placeholder.buffer();
        let tag_table = buffer.tag_table();

        let color = gdk4::RGBA::new(color.r, color.g, color.b, color.a);

        tag_table.foreach(|tag| {
            tag.set_foreground_rgba(Some(&color));

            buffer.apply_tag(
                tag,
                &buffer.start_iter(),
                &buffer.end_iter(),
            );
        });
    }

    fn set_text(&mut self, _platform: &mut Platform, text: String) {
        self.view.buffer().set_text(&text);
    }

    fn set_placeholder_text(&mut self, _platform: &mut Platform, text: String) {
        let buffer = self.placeholder.buffer();
        let tag_table = buffer.tag_table();

        tag_table.foreach(|tag| {
            buffer.apply_tag(
                tag,
                &buffer.start_iter(),
                &buffer.end_iter(),
            );
        });

        buffer.set_text(&text);
    }

    fn get_measureable(&mut self, _platform: &mut Platform) -> impl Measurable<Platform> {
        Layout {
            view: self.view.clone(),
        }
    }
}

struct Layout {
    view: gtk4::TextView,
}

impl Measurable<Platform> for Layout {
    fn measure(
        &mut self,
        _platform: &mut Platform,
        known_size: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> (Size<f32>, Option<f32>) {
        let context = self.view.pango_context();
        let metrics = context.metrics(None, None);
        let theight = metrics.height() as f32 / pango::SCALE as f32;
        let tascent = metrics.ascent() as f32 / pango::SCALE as f32;

        let size = Size {
            width:  known_size.width.unwrap_or(0.0),
            height: theight.ceil(),
        };

        (size, Some(tascent.ceil()))
    }
}

fn font_style(font: &Font) -> String {
    let family = font.family.as_ref().map_or(String::new(), |family| {
        format!("font-family: \"{}\";", family)
    });

    let strikethrough = if font.striketrough {
        "text-decoration: line-through;"
    } else {
        ""
    };

    let stretch = match font.stretch {
        Stretch::UltraCondensed => "ultra-condensed",
        Stretch::ExtraCondensed => "extra-condensed",
        Stretch::Condensed => "condensed",
        Stretch::SemiCondensed => "semi-condensed",
        Stretch::Normal => "normal",
        Stretch::SemiExpanded => "semi-expanded",
        Stretch::Expanded => "expanded",
        Stretch::ExtraExpanded => "extra-expanded",
        Stretch::UltraExpanded => "ultra-expanded",
    };

    let style = match font.italic {
        true => "italic",
        false => "normal",
    };

    let size = format!(
        "font-size: {}px;",
        font.size * (96.0 / 72.0),
    );
    let weight = format!("font-weight: {};", font.weight.0);
    let stretch = format!("font-stretch: {stretch};");
    let style = format!("font-style: {style};");
    let color = format!(
        "color: rgba({}, {}, {}, {});",
        font.color.r * 255.0,
        font.color.g * 255.0,
        font.color.b * 255.0,
        font.color.a,
    );

    family + &size + &weight + &stretch + &style + strikethrough + &color
}
