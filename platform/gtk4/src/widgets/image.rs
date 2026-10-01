use std::{borrow::Cow, io};

use gdk4::{gdk_pixbuf::prelude::PixbufLoaderExt, prelude::PaintableExt};
use glib::{object::Cast, subclass::types::ObjectSubclassIsExt};
use librsvg::prelude::HandleExt;
use ori_native_core::{AvailableSpace, Color, Measurable, Size, native::NativeImage};

use crate::Platform;

pub struct Image {
    picture:   gtk4::Picture,
    paintable: Option<Paintable>,
    tint:      Option<Color>,
}

impl NativeImage<Platform> for Image {
    type Error = io::Error;

    fn build(_platform: &mut Platform) -> Self {
        let picture = gtk4::Picture::new();

        Self {
            picture,
            paintable: None,
            tint: None,
        }
    }

    fn teardown(self, _platform: &mut Platform) {}

    fn widget_ref(&self) -> gtk4::Widget {
        self.picture.clone().upcast()
    }

    fn load_data(
        &mut self,
        _platform: &mut Platform,
        data: Cow<'static, [u8]>,
    ) -> Result<impl Measurable<Platform>, Self::Error> {
        let paintable = Paintable::new(&data)?;
        paintable.set_tint(self.tint);

        self.picture.set_paintable(Some(&paintable));
        self.paintable = Some(paintable.clone());

        Ok(Layout { paintable })
    }

    fn set_tint(&mut self, _platform: &mut Platform, tint: Option<Color>) {
        self.tint = tint;

        if let Some(ref paintable) = self.paintable {
            paintable.set_tint(tint);
        }
    }
}

struct Layout {
    paintable: Paintable,
}

impl Measurable<Platform> for Layout {
    fn measure(
        &mut self,
        _platform: &mut Platform,
        known_size: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> (Size<f32>, Option<f32>) {
        let (width, height) = self.paintable.intrinsic_size().unwrap_or((0.0, 0.0));

        let width = match available_space.width {
            AvailableSpace::Definite(available) => available.min(width as f32),
            AvailableSpace::MinContent => 0.0,
            AvailableSpace::MaxContent => width as f32,
        };

        let height = match available_space.height {
            AvailableSpace::Definite(available) => available.min(height as f32),
            AvailableSpace::MinContent => 0.0,
            AvailableSpace::MaxContent => height as f32,
        };

        let size = Size {
            width:  known_size.width.unwrap_or(width),
            height: known_size.height.unwrap_or(height),
        };

        (size, None)
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
        if data.windows(4).take(64).any(|w| w == b"<svg") {
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
        let handle = librsvg::Handle::from_data(data).map_err(io::Error::other)?;

        let this: Self = glib::Object::builder().build();
        this.imp().handle.replace(Contents::Svg(handle));

        Ok(this)
    }

    fn set_tint(&self, tint: Option<Color>) {
        self.imp().tint.replace(tint);
        self.invalidate_contents();
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
