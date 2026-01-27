use gtk4::prelude::{TextBufferExt, TextBufferExtManual, TextTagExt, TextViewExt, WidgetExt};
use ori_native_core::{
    LayoutLeaf, Stretch, TextSpan, Weight,
    native::{HasText, NativeText},
};

use crate::{Platform, platform::StyleNode};

pub struct Text {
    view:  gtk4::TextView,
    style: StyleNode,
}

impl NativeText<Platform> for Text {
    type Leaf = TextLeaf;

    fn widget(&self) -> &gtk4::Widget {
        self.view.as_ref()
    }

    fn build(platform: &mut Platform, spans: Box<[TextSpan]>, text: String) -> (Self, Self::Leaf) {
        let view = gtk4::TextView::new();
        view.set_editable(false);
        view.set_cursor_visible(false);
        view.set_sensitive(false);

        let style = platform.add_style("background: none;");
        view.set_css_classes(&[&style.class()]);

        let mut this = Self { view, style };
        let leaf = this.set_text(spans, text);

        (this, leaf)
    }

    fn teardown(self, platform: &mut Platform) {
        platform.remove_style(self.style);
    }

    fn set_text(&mut self, spans: Box<[TextSpan]>, text: String) -> Self::Leaf {
        let buffer = self.view.buffer();
        let mut iter = buffer.start_iter();

        for span in &spans {
            let tag = gtk4::TextTag::new(None);
            tag.set_size((span.attributes.size * pango::SCALE as f32) as i32);
            tag.set_family(span.attributes.family.as_deref());
            tag.set_weight(span.attributes.weight.0 as i32);
            tag.set_stretch(convert_stretch(span.attributes.stretch));
            tag.set_style(match span.attributes.italic {
                false => pango::Style::Normal,
                true => pango::Style::Italic,
            });

            buffer.tag_table().add(&tag);
            buffer.insert_with_tags(&mut iter, &text, &[&tag]);
        }

        TextLeaf {
            view: self.view.clone(),
            spans,
            text,
        }
    }
}

impl HasText for Platform {
    type Text = Text;
}

pub struct TextLeaf {
    view:  gtk4::TextView,
    spans: Box<[TextSpan]>,
    text:  String,
}

impl LayoutLeaf<Platform> for TextLeaf {
    fn measure(
        &mut self,
        _platform: &mut Platform,
        _known_size: taffy::Size<Option<f32>>,
        _available_space: taffy::Size<taffy::AvailableSpace>,
    ) -> taffy::Size<f32> {
        let context = self.view.pango_context();
        let layout = pango::Layout::new(&context);

        layout.set_text(&self.text);

        let attrs = pango::AttrList::new();

        for span in &self.spans {
            let mut desc = pango::FontDescription::new();

            if let Some(ref family) = span.attributes.family {
                desc.set_family(family);
            }

            desc.set_size((span.attributes.size * pango::SCALE as f32) as i32);
            desc.set_weight(convert_weight(span.attributes.weight));
            desc.set_stretch(convert_stretch(span.attributes.stretch));
            desc.set_style(match span.attributes.italic {
                false => pango::Style::Normal,
                true => pango::Style::Italic,
            });

            let mut attr = pango::AttrFontDesc::new(&desc);
            attr.set_start_index(span.range.start as u32);
            attr.set_end_index(span.range.end as u32);

            attrs.insert(attr);
        }

        layout.set_attributes(Some(&attrs));

        let (width, height) = layout.pixel_size();

        taffy::Size {
            width:  width as f32,
            height: height as f32,
        }
    }
}

fn convert_weight(weight: Weight) -> pango::Weight {
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

fn convert_stretch(stretch: Stretch) -> pango::Stretch {
    match stretch {
        Stretch::UltraCondensed => pango::Stretch::UltraCondensed,
        Stretch::ExtraCondensed => pango::Stretch::ExtraCondensed,
        Stretch::Condensed => pango::Stretch::Condensed,
        Stretch::SemiCondensed => pango::Stretch::SemiCondensed,
        Stretch::Normal => pango::Stretch::Normal,
        Stretch::SemiExpanded => pango::Stretch::SemiExpanded,
        Stretch::Expanded => pango::Stretch::Expanded,
        Stretch::ExtraExpanded => pango::Stretch::ExtraExpanded,
        Stretch::UntraExpanded => pango::Stretch::UltraExpanded,
    }
}
