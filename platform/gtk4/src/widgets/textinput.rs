use std::{cell::Cell, rc::Rc};

use glib::object::{Cast, ObjectExt};
use gtk4::prelude::{TextBufferExt, TextViewExt, WidgetExt};
use ori_native_core::{
    AvailableSpace, Font, Measurable, Newline, Size, Stretch, TextAlign, TextWrap,
    native::NativeTextInput,
};

use crate::{Platform, platform::StyleNode};

pub struct TextInput {
    overlay:     gtk4::Overlay,
    view:        gtk4::TextView,
    placeholder: gtk4::TextView,

    view_style: StyleNode,

    font:             Font,
    placeholder_font: Font,
    newline:          Rc<Cell<Newline>>,
}

impl NativeTextInput<Platform> for TextInput {
    fn build(
        platform: &mut Platform,
        on_change: impl Fn(String) + 'static,
        on_submit: impl Fn(String) + 'static,
    ) -> Self {
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

        let controller = gtk4::EventControllerFocus::new();
        controller.connect_enter({
            let placeholder = placeholder.clone();

            move |_| {
                placeholder.set_visible(false);
            }
        });

        controller.connect_leave({
            let view = view.downgrade();
            let placeholder = placeholder.clone();

            move |_| {
                if let Some(view) = view.upgrade()
                    && view.buffer().start_iter() == view.buffer().end_iter()
                {
                    placeholder.set_visible(true);
                }
            }
        });

        view.add_controller(controller);

        view.buffer().connect_text_notify({
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

                on_change(text.into());
            }
        });

        let controller = gtk4::EventControllerKey::new();

        controller.connect_key_pressed({
            let enter = newline.clone();
            let buffer = view.buffer();

            move |_, key, _, state| {
                let shift = state.contains(gdk4::ModifierType::SHIFT_MASK);

                let can_submit = match enter.get() {
                    Newline::None => true,
                    Newline::ShiftEnter if !shift => true,
                    _ => false,
                };

                if key == gdk4::Key::Return && can_submit {
                    let text = buffer.text(
                        &buffer.start_iter(),
                        &buffer.end_iter(),
                        true,
                    );

                    on_submit(text.into());

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
            placeholder_font: Default::default(),
            newline,
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

    fn set_accept_tab(&mut self, _platform: &mut Platform, accept_tab: bool) {
        self.view.set_accepts_tab(accept_tab);
    }

    fn set_font(&mut self, platform: &mut Platform, font: Font, align: TextAlign, wrap: TextWrap) {
        self.view.set_justification(match align {
            TextAlign::Start => gtk4::Justification::Left,
            TextAlign::Center => gtk4::Justification::Center,
            TextAlign::End => gtk4::Justification::Right,
            TextAlign::Justify => gtk4::Justification::Fill,
        });

        self.view.set_wrap_mode(match wrap {
            TextWrap::Word => gtk4::WrapMode::Word,
            TextWrap::Char => gtk4::WrapMode::Char,
            TextWrap::None => gtk4::WrapMode::None,
        });

        platform.set_style(self.view_style, &font_style(&font));
        self.font = font;
    }

    fn set_placeholder_font(
        &mut self,
        _platform: &mut Platform,
        font: Font,
        align: TextAlign,
        wrap: TextWrap,
    ) {
        self.placeholder.set_visible(true);

        self.placeholder.set_justification(match align {
            TextAlign::Start => gtk4::Justification::Left,
            TextAlign::Center => gtk4::Justification::Center,
            TextAlign::End => gtk4::Justification::Right,
            TextAlign::Justify => gtk4::Justification::Fill,
        });

        self.placeholder.set_wrap_mode(match wrap {
            TextWrap::Word => gtk4::WrapMode::Word,
            TextWrap::Char => gtk4::WrapMode::Char,
            TextWrap::None => gtk4::WrapMode::None,
        });

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

        self.placeholder_font = font;
    }

    fn set_text(&mut self, _platform: &mut Platform, text: String) {
        self.view.buffer().set_text(&text);
    }

    fn set_placeholder_text(&mut self, _platform: &mut Platform, text: String) {
        let buffer = self.placeholder.buffer();
        let tag_table = buffer.tag_table();
        let tag = super::text::font_tag(&self.placeholder_font);

        tag_table.foreach(|tag| tag_table.remove(tag));
        tag_table.add(&tag);

        buffer.set_text(&text);
        buffer.apply_tag(
            &tag,
            &buffer.start_iter(),
            &buffer.end_iter(),
        );
    }

    fn get_measureable(&mut self, _platform: &mut Platform) -> impl Measurable<Platform> {
        Layout {
            view:             self.view.clone(),
            font:             self.font.clone(),
            placeholder_font: self.placeholder_font.clone(),
        }
    }
}

struct Layout {
    view:             gtk4::TextView,
    font:             Font,
    placeholder_font: Font,
}

impl Measurable<Platform> for Layout {
    fn measure(
        &mut self,
        _platform: &mut Platform,
        known_size: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        let context = self.view.pango_context();

        let desc = super::text::font_description(&self.font);
        let metrics = context.metrics(Some(&desc), context.language().as_ref());
        let theight = (metrics.ascent() + metrics.descent()) as f32 / pango::SCALE as f32;

        let desc = super::text::font_description(&self.placeholder_font);
        let metrics = context.metrics(Some(&desc), context.language().as_ref());
        let pheight = (metrics.ascent() + metrics.descent()) as f32 / pango::SCALE as f32;

        Size {
            width:  known_size.width.unwrap_or(0.0),
            height: (theight.max(pheight) * 1.25).ceil(),
        }
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

    let size = format!("font-size: {}pt;", font.size);
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
