use crate::pixmap::Pixmap;
use gtk::{gdk, glib};

glib::wrapper! {
    pub struct PaintablePixmap(ObjectSubclass<imp::PaintablePixmap>)
        @implements gdk::Paintable;
}

impl PaintablePixmap {
    pub fn new() -> Self {
        let result = glib::Object::new(&[]).expect("Failed to create `PaintablePixmap`.");
        log::info!("{:?}", result);
        result
    }
}

mod imp {
    use crate::pixmap::{InMemoryPixmap, Pixmap};
    use gtk::glib::ParamSpec;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{gdk, glib, graphene, gsk};
    use once_cell::sync::Lazy;

    // Object holding the state
    #[derive(Default, Debug)]
    pub struct PaintablePixmap(InMemoryPixmap);

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for PaintablePixmap {
        const NAME: &'static str = "PixelflutPaintablePixmap";
        type Type = super::PaintablePixmap;
        type Interfaces = (gdk::Paintable,);
    }

    // Trait shared by all GObjects
    impl ObjectImpl for PaintablePixmap {}

    // Trait shared by all Paintables
    impl PaintableImpl for PaintablePixmap {
        fn flags(&self, _paintable: &Self::Type) -> gdk::PaintableFlags {
            // fixed size
            gdk::PaintableFlags::SIZE
        }

        fn intrinsic_width(&self, _paintable: &Self::Type) -> i32 {
            self.0.get_size().expect("Could not get size of pixmap").0 as i32
        }

        fn intrinsic_height(&self, _paintable: &Self::Type) -> i32 {
            self.0.get_size().expect("Could not get size of pixmap").1 as i32
        }

        fn intrinsic_aspect_ratio(&self, _paintable: &Self::Type) -> f64 {
            let size = self.0.get_size().expect("Could not get size of pixmap");
            (size.0 as f64) / (size.1 as f64)
        }

        fn snapshot(&self, _paintable: &Self::Type, snapshot: &gdk::Snapshot, width: f64, height: f64) {
            let snapshot = snapshot.downcast_ref::<gtk::Snapshot>().unwrap();

            for iy in 0..height as usize {
                for ix in 0..width as usize {
                    let pixel = self
                        .0
                        .get_pixel(ix, iy)
                        .expect("Could not get pixel value from pixmap");
                    snapshot.append_color(
                        &gdk::RGBA::new(pixel.0 as f32, pixel.1 as f32, pixel.2 as f32, 1f32),
                        &graphene::Rect::new(ix as f32, iy as f32, 1f32, 1f32),
                    )
                }
            }
        }
    }
}
