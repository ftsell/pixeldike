//! PixmapDisplay displays a [`GDK Pixbuf`](gdk_pixbuf::auto::pixbuf)

use crate::gui::components::server::server_layout::ServerLayoutModel;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib::Sender;
use gtk::prelude::WidgetExt;
use gtk::Align;
use pixelflut::pixmap::gdk_pixbuf_pixmap::default_gdk_pixbuf_pixmap;
use pixelflut::pixmap::{Color, Pixmap};
use relm4::{ComponentUpdate, Model, Widgets};

type ParentModel = ServerLayoutModel;

/// State of the *PixmapDisplay* component
pub(in crate::gui) struct PixmapDisplayModel {
    pixbuf: Pixbuf,
    visibility: bool,
}

/// State altering messages of the *PixmapDisplay* component
pub(in crate::gui) enum PixmapDisplayMsg {
    SetVisibility(bool),
    UpdatePixmapData(Box<Vec<Color>>),
}

/// GTK widgets used to render the *PixmapDisplay* component
pub(in crate::gui) struct PixmapDisplayWidgets {
    picture: gtk::Picture,
}

impl Model for PixmapDisplayModel {
    type Msg = PixmapDisplayMsg;
    type Widgets = PixmapDisplayWidgets;
    type Components = ();
}

impl ComponentUpdate<ParentModel> for PixmapDisplayModel {
    fn init_model(_parent_model: &ParentModel) -> Self {
        Self {
            pixbuf: default_gdk_pixbuf_pixmap(),
            visibility: false,
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        _sender: Sender<Self::Msg>,
        _parent_sender: Sender<<ParentModel as Model>::Msg>,
    ) {
        match msg {
            PixmapDisplayMsg::UpdatePixmapData(data) => {
                self.pixbuf.put_raw_data(&data).expect("Could not put raw data");
            }
            PixmapDisplayMsg::SetVisibility(value) => {
                self.visibility = value;
            }
        }
    }
}

impl Widgets<PixmapDisplayModel, ParentModel> for PixmapDisplayWidgets {
    type Root = gtk::Picture;

    fn init_view(
        model: &PixmapDisplayModel,
        _components: &<PixmapDisplayModel as Model>::Components,
        _sender: Sender<<PixmapDisplayModel as Model>::Msg>,
    ) -> Self {
        let picture = gtk::Picture::builder()
            .name("PixelflutPixmapDisplay")
            .halign(Align::Center)
            .valign(Align::Center)
            .visible(model.visibility)
            .build();
        picture.set_pixbuf(Some(&model.pixbuf));

        Self { picture }
    }

    fn root_widget(&self) -> Self::Root {
        self.picture.clone()
    }

    fn view(&mut self, model: &PixmapDisplayModel, _sender: Sender<<PixmapDisplayModel as Model>::Msg>) {
        self.picture.set_visible(model.visibility);
        self.picture.set_pixbuf(Some(&model.pixbuf));
    }
}
