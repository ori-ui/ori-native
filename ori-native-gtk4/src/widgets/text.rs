use gtk4::prelude::{TextBufferExt, TextBufferExtManual, TextTagExt, TextViewExt, WidgetExt};
use ori_native_core::{
    Font, LayoutLeaf, NativeWidget, Stretch, TextSpan, Weight, Wrap,
    native::{HasText, NativeText},
};

use crate::Platform;

impl HasText for Platform {
    type Text = Text;
}

pub struct Text {
    view: gtk4::TextView,
}

impl NativeWidget<Platform> for Text {
    fn widget(&self) -> &gtk4::Widget {
        self.view.as_ref()
    }
}

impl NativeText<Platform> for Text {
    type Layout = TextLayout;

    fn build(
        _platform: &mut Platform,
        spans: Box<[TextSpan]>,
        text: String,
        wrap: Wrap,
    ) -> (Self, Self::Layout) {
        let view = gtk4::TextView::new();
        view.set_editable(false);
        view.set_cursor_visible(false);
        view.set_sensitive(false);

        let mut this = Self { view };
        let leaf = this.set_text(spans, text, wrap);

        (this, leaf)
    }

    fn teardown(self, _platform: &mut Platform) {}

    fn set_text(&mut self, spans: Box<[TextSpan]>, text: String, wrap: Wrap) -> Self::Layout {
        match wrap {
            Wrap::Word => self.view.set_wrap_mode(gtk4::WrapMode::Word),
            Wrap::Char => self.view.set_wrap_mode(gtk4::WrapMode::Char),
            Wrap::None => self.view.set_wrap_mode(gtk4::WrapMode::None),
        }

        let buffer = self.view.buffer();
        buffer.set_text("");

        let tag_table = buffer.tag_table();
        let mut iter = buffer.start_iter();

        tag_table.foreach(|tag| tag_table.remove(tag));

        for span in &spans {
            let tag = font_tag(&span.font);
            tag_table.add(&tag);
            buffer.insert_with_tags(&mut iter, &text, &[&tag]);
        }

        TextLayout {
            view: self.view.clone(),
            spans,
            text,
            wrap,
        }
    }
}

pub struct TextLayout {
    view:  gtk4::TextView,
    spans: Box<[TextSpan]>,
    text:  String,
    wrap:  Wrap,
}

impl LayoutLeaf<Platform> for TextLayout {
    fn measure(
        &mut self,
        _platform: &mut Platform,
        known_size: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
    ) -> taffy::Size<f32> {
        let context = self.view.pango_context();
        let layout = pango::Layout::new(&context);

        layout.set_text(&self.text);

        let attrs = pango::AttrList::new();

        let mut min_height: f32 = 0.0;

        for span in &self.spans {
            let desc = font_description(&span.font);
            let mut attr = pango::AttrFontDesc::new(&desc);
            attr.set_start_index(span.range.start as u32);
            attr.set_end_index(span.range.end as u32);

            attrs.insert(attr);

            let metrics = context.metrics(Some(&desc), context.language().as_ref());
            let height = (metrics.ascent() + metrics.descent()) as f32 / pango::SCALE as f32;
            min_height = min_height.max(height);
        }

        layout.set_attributes(Some(&attrs));

        match self.wrap {
            Wrap::Word => layout.set_wrap(pango::WrapMode::Word),
            Wrap::Char => layout.set_wrap(pango::WrapMode::Char),
            Wrap::None => {}
        }

        if !matches!(self.wrap, Wrap::None) {
            match available_space.width {
                taffy::AvailableSpace::MinContent => layout.set_width(0),
                taffy::AvailableSpace::MaxContent => layout.set_width(-1),
                taffy::AvailableSpace::Definite(width) => {
                    layout.set_width((width * pango::SCALE as f32).round() as i32);
                }
            }
        }

        let (width, height) = layout.pixel_size();

        taffy::Size {
            width:  known_size.width.unwrap_or(width as f32),
            height: min_height.max(height as f32),
        }
    }
}

pub(super) fn font_tag(font: &Font) -> gtk4::TextTag {
    let tag = gtk4::TextTag::new(None);
    tag.set_size((font.size * pango::SCALE as f32).round() as i32);
    tag.set_family(font.family.as_deref());
    tag.set_weight(font.weight.0 as i32);
    tag.set_stretch(convert_stretch(font.stretch));

    tag.set_style(match font.italic {
        false => pango::Style::Normal,
        true => pango::Style::Italic,
    });

    let color = gdk4::RGBA::new(
        font.color.r,
        font.color.g,
        font.color.b,
        font.color.a,
    );

    if font.striketrough {
        tag.set_strikethrough(true);
        tag.set_strikethrough_rgba(Some(&color));
    }

    tag.set_foreground_rgba(Some(&color));

    tag
}

pub(super) fn font_description(font: &Font) -> pango::FontDescription {
    let mut desc = pango::FontDescription::new();

    if let Some(ref family) = font.family {
        desc.set_family(family);
    }

    desc.set_size((font.size * pango::SCALE as f32) as i32);
    desc.set_weight(convert_weight(font.weight));
    desc.set_stretch(convert_stretch(font.stretch));
    desc.set_style(match font.italic {
        false => pango::Style::Normal,
        true => pango::Style::Italic,
    });

    desc
}

pub(super) fn convert_weight(weight: Weight) -> pango::Weight {
    match weight {
        Weight(100) => pango::Weight::Thin,
        Weight(200) => pango::Weight::Ultralight,
        Weight(300) => pango::Weight::Light,
        Weight(350) => pango::Weight::Semilight,
        Weight(380) => pango::Weight::Book,
        Weight(400) => pango::Weight::Normal,
        Weight(500) => pango::Weight::Medium,
        Weight(600) => pango::Weight::Semibold,
        Weight(700) => pango::Weight::Bold,
        Weight(800) => pango::Weight::Ultrabold,
        Weight(900) => pango::Weight::Heavy,
        Weight(1000) => pango::Weight::Ultraheavy,
        Weight(..) => pango::Weight::Normal,
    }
}

pub(super) fn convert_stretch(stretch: Stretch) -> pango::Stretch {
    match stretch {
        Stretch::UltraCondensed => pango::Stretch::UltraCondensed,
        Stretch::ExtraCondensed => pango::Stretch::ExtraCondensed,
        Stretch::Condensed => pango::Stretch::Condensed,
        Stretch::SemiCondensed => pango::Stretch::SemiCondensed,
        Stretch::Normal => pango::Stretch::Normal,
        Stretch::SemiExpanded => pango::Stretch::SemiExpanded,
        Stretch::Expanded => pango::Stretch::Expanded,
        Stretch::ExtraExpanded => pango::Stretch::ExtraExpanded,
        Stretch::UltraExpanded => pango::Stretch::UltraExpanded,
    }
}
