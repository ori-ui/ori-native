use std::{borrow::Cow, io};

use gdk4::{gdk_pixbuf::prelude::PixbufLoaderExt, prelude::PaintableExt};
use glib::subclass::types::ObjectSubclassIsExt;
use librsvg::prelude::HandleExt;
use ori_native_core::{
    Color, LayoutLeaf, NativeWidget,
    native::{HasImage, NativeImage},
};

use crate::Platform;

impl HasImage for Platform {
    type Image = Image;
}

pub struct Image {
    image: gtk4::Picture,
    svg:   Option<Paintable>,
    tint:  Option<Color>,
}

impl NativeWidget<Platform> for Image {
    fn widget(&self) -> &gtk4::Widget {
        self.image.as_ref()
    }
}

impl NativeImage<Platform> for Image {
    type Error = io::Error;

    fn build(_plaform: &mut Platform) -> Self {
        let image = gtk4::Picture::new();

        Self {
            image,
            svg: None,
            tint: None,
        }
    }

    fn teardown(self, _plaform: &mut Platform) {}

    fn load_data(
        &mut self,
        _plaform: &mut Platform,
        data: Cow<'static, [u8]>,
    ) -> Result<impl LayoutLeaf<Platform>, Self::Error> {
        let paintable = Paintable::new(&data)?;
        paintable.set_tint(self.tint);
        self.image.set_paintable(Some(&paintable));

        Ok(Layout { paintable })
    }

    fn set_tint(&mut self, tint: Option<Color>) {
        self.tint = tint;

        if let Some(ref svg) = self.svg {
            svg.set_tint(tint);
        }
    }
}

struct Layout {
    paintable: Paintable,
}

impl LayoutLeaf<Platform> for Layout {
    fn measure(
        &mut self,
        _platform: &mut Platform,
        known_size: taffy::Size<Option<f32>>,
        _available_space: taffy::Size<taffy::AvailableSpace>,
    ) -> taffy::Size<f32> {
        let (width, height) = self.paintable.intrinsic_size().unwrap_or((0.0, 0.0));

        taffy::Size {
            width:  known_size.width.unwrap_or(width as f32),
            height: known_size.height.unwrap_or(height as f32),
        }
    }
}

#[derive(Default)]
enum Contents {
    Svg(librsvg::Handle),
    Texture(gdk4::Texture),
    #[default]
    None,
}

glib::wrapper! {
    struct Paintable(ObjectSubclass<imp::Paintable>)
        @implements
            gdk4::Paintable;
}

impl Paintable {
    fn new(data: &[u8]) -> io::Result<Self> {
        if data.starts_with(&[0x3c, 0x3f, 0x78, 0x6d, 0x6c]) {
            Self::new_svg(data)
        } else {
            Self::new_texture(data)
        }
    }

    fn new_texture(data: &[u8]) -> io::Result<Self> {
        let loader = gdk4::gdk_pixbuf::PixbufLoader::new();
        loader.write(data).map_err(io::Error::other)?;
        loader.close().map_err(io::Error::other)?;

        let pixbuf = loader
            .pixbuf()
            .ok_or_else(|| io::Error::other("no pixbuf"))?;

        let texture = gdk4::Texture::for_pixbuf(&pixbuf);

        let this: Self = glib::Object::builder().build();
        this.imp().handle.replace(Contents::Texture(texture));

        Ok(this)
    }

    fn new_svg(data: &[u8]) -> io::Result<Self> {
        let handle = librsvg::Handle::from_data(data)
            .map_err(io::Error::other)?
            .ok_or_else(|| io::Error::other("no handle"))?;

        let this: Self = glib::Object::builder().build();
        this.imp().handle.replace(Contents::Svg(handle));

        Ok(this)
    }

    fn set_tint(&self, tint: Option<Color>) {
        if self.imp().tint.replace(tint) != tint {
            self.invalidate_contents();
        }
    }

    fn intrinsic_size(&self) -> Option<(f64, f64)> {
        match *self.imp().handle.borrow() {
            Contents::Svg(ref handle) => handle.intrinsic_size_in_pixels(),

            Contents::Texture(ref texture) => Some((
                texture.intrinsic_width() as f64,
                texture.intrinsic_height() as f64,
            )),

            Contents::None => None,
        }
    }
}

mod imp {
    use std::cell::{Cell, RefCell};

    use gdk4::subclass::prelude::PaintableImpl;
    use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
    use gtk4::prelude::SnapshotExt;
    use librsvg::prelude::HandleExt;
    use ori_native_core::Color;

    use super::Contents;

    #[derive(Default)]
    pub(super) struct Paintable {
        pub(super) handle: RefCell<Contents>,
        pub(super) tint:   Cell<Option<Color>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Paintable {
        const NAME: &'static str = "OriSvg";

        type Type = super::Paintable;
        type ParentType = glib::Object;
        type Interfaces = (gdk4::Paintable,);
    }

    impl ObjectImpl for Paintable {}

    impl PaintableImpl for Paintable {
        fn snapshot(&self, snapshot: &gdk4::Snapshot, width: f64, height: f64) {
            let cr = snapshot.append_cairo(&graphene::Rect::new(
                0.0,
                0.0,
                width as f32,
                height as f32,
            ));

            if self.tint.get().is_some() {
                cr.push_group();
            }

            match *self.handle.borrow() {
                Contents::Svg(ref handle) => {
                    let bounds = librsvg::Rectangle::new(0.0, 0.0, width, height);
                    let _ = handle.render_document(&cr, &bounds);
                }

                Contents::Texture(ref texture) => {
                    let bounds = graphene::Rect::new(0.0, 0.0, width as f32, height as f32);
                    snapshot.append_texture(texture, &bounds);
                }

                Contents::None => {}
            }

            if let Some(tint) = self.tint.get()
                && let Ok(mask) = cr.pop_group()
            {
                cr.set_source_rgba(
                    tint.r as f64,
                    tint.g as f64,
                    tint.b as f64,
                    tint.a as f64,
                );

                let _ = cr.mask(&mask);
            }
        }
    }
}
