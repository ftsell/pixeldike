//! PixmapDisplay displays a [`GDK Pixbuf`](gdk_pixbuf::auto::pixbuf)

use crate::gui::components::server::server_layout::ServerLayoutModel;
use gtk::glib::Sender;
use gtk::traits::WidgetExt;
use gtk::{Align, IconSize};
use relm4::{ComponentUpdate, Model, Widgets};

type ParentModel = ServerLayoutModel;

/// State of the *PixmapDisplay* component
pub(in crate::gui) struct PixmapDisplayModel {
    pixbuf: Option<gtk::gdk_pixbuf::Pixbuf>,
}

/// State altering messages of the *PixmapDisplay* component
pub(in crate::gui) enum PixmapDisplayMsg {
    SetPixbuf(Option<gtk::gdk_pixbuf::Pixbuf>),
}

/// GTK widgets used to render the *PixmapDisplay* component
pub(in crate::gui) struct PixmapDisplayWidgets {
    image: gtk::Image,
}

impl Model for PixmapDisplayModel {
    type Msg = PixmapDisplayMsg;
    type Widgets = PixmapDisplayWidgets;
    type Components = ();
}

impl ComponentUpdate<ParentModel> for PixmapDisplayModel {
    fn init_model(_parent_model: &ParentModel) -> Self {
        Self { pixbuf: None }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        _sender: Sender<Self::Msg>,
        _parent_sender: Sender<<ParentModel as Model>::Msg>,
    ) {
        match msg {
            PixmapDisplayMsg::SetPixbuf(msg) => {
                log::debug!("Setting pixbuf to {:?}", msg);
                self.pixbuf = msg;
            }
        }
    }
}

impl Widgets<PixmapDisplayModel, ParentModel> for PixmapDisplayWidgets {
    type Root = gtk::Image;

    fn init_view(
        _model: &PixmapDisplayModel,
        _components: &<PixmapDisplayModel as Model>::Components,
        _sender: Sender<<PixmapDisplayModel as Model>::Msg>,
    ) -> Self {
        let image = gtk::Image::builder()
            .name("pixmap")
            .halign(Align::Center)
            .valign(Align::Center)
            .icon_size(IconSize::Large)
            .hexpand(true)
            .vexpand(true)
            .visible(false)
            .build();

        Self { image }
    }

    fn root_widget(&self) -> Self::Root {
        self.image.clone()
    }

    fn view(&mut self, model: &PixmapDisplayModel, _sender: Sender<<PixmapDisplayModel as Model>::Msg>) {
        match &model.pixbuf {
            None => {
                self.image.clear();
                self.image.set_visible(false);
            }
            Some(pixbuf) => {
                self.image.set_from_gicon(&pixbuf.clone());
                self.image.set_visible(true);
            }
        }
    }
}
