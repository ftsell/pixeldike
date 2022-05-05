//! ServerLayout is a component that organizes widgets and sub-components for a server GUI

use gtk::glib::Sender;
use gtk::prelude::*;
use gtk::{Align, Overflow};
use relm4::{send, ComponentUpdate, Model, RelmComponent, WidgetPlus, Widgets};

use crate::gui::components::server::pixmap_display::{PixmapDisplayModel, PixmapDisplayMsg};
use crate::gui::components::server::server_config_form::{ConfigFormModel, ProtocolChoice};
use crate::gui::components::server::server_holder::{ServerHolderModel, ServerHolderMsg};

type ParentModel = ServerHolderModel;

/// State of the *ServerLayout* component
pub(in crate::gui) struct ServerLayoutModel {
    is_server_running: bool,
}

/// State altering messages of the *ServerLayout* component
pub(in crate::gui) enum ServerLayoutMsg {
    StartServer { protocol: ProtocolChoice, port: u32 },
    StopServer,
    SetPixbuf(Option<gtk::gdk_pixbuf::Pixbuf>),
}

/// GTK widgets that are directly used to render the *ServerLayout* component
#[allow(dead_code)]
pub(in crate::gui) struct ServerLayoutWidgets {
    layout_box: gtk::Box,
    separator: gtk::Separator,
    server_not_running_text: gtk::Text,
}

/// Sub-Components used by the *ServerLayout* component
#[derive(relm4::Components)]
pub(in crate::gui) struct ServerLayoutComponents {
    config_form: RelmComponent<ConfigFormModel, ServerLayoutModel>,
    pixmap_display: RelmComponent<PixmapDisplayModel, ServerLayoutModel>,
}

impl Model for ServerLayoutModel {
    type Msg = ServerLayoutMsg;
    type Widgets = ServerLayoutWidgets;
    type Components = ServerLayoutComponents;
}

impl ComponentUpdate<ParentModel> for ServerLayoutModel {
    fn init_model(_parent_model: &ParentModel) -> Self {
        Self {
            is_server_running: false,
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        _sender: Sender<Self::Msg>,
        parent_sender: Sender<<ParentModel as Model>::Msg>,
    ) {
        match msg {
            ServerLayoutMsg::StartServer { port, protocol } => {
                self.is_server_running = true;
                send!(parent_sender, ServerHolderMsg::StartServer { protocol, port });
            }
            ServerLayoutMsg::SetPixbuf(pixbuf) => {
                send!(components.pixmap_display, PixmapDisplayMsg::SetPixbuf(pixbuf));
            }
            ServerLayoutMsg::StopServer => {
                self.is_server_running = false;
                send!(parent_sender, ServerHolderMsg::StopServer);
                send!(components.pixmap_display, PixmapDisplayMsg::SetPixbuf(None));
            }
        }
    }
}

impl Widgets<ServerLayoutModel, ParentModel> for ServerLayoutWidgets {
    type Root = gtk::Box;

    fn init_view(
        _model: &ServerLayoutModel,
        components: &<ServerLayoutModel as Model>::Components,
        _sender: Sender<<ServerLayoutModel as Model>::Msg>,
    ) -> Self {
        let layout_box = gtk::Box::builder()
            .name("PixelflutLayout")
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .build();
        layout_box.set_margin_all(4);

        let separator = gtk::Separator::builder()
            .name("PixelflutLayoutSeparator")
            .orientation(gtk::Orientation::Vertical)
            .build();

        let server_not_running_text = gtk::Text::builder()
            .text("Server is not currently running")
            .valign(Align::Center)
            .halign(Align::Center)
            .vexpand(true)
            .hexpand(true)
            .visible(true)
            .editable(false)
            .sensitive(false)
            .overflow(Overflow::Visible)
            .build();

        layout_box.append(components.config_form.root_widget());
        layout_box.append(&separator);
        layout_box.append(components.pixmap_display.root_widget());
        layout_box.append(&server_not_running_text);

        Self {
            layout_box,
            separator,
            server_not_running_text,
        }
    }

    fn root_widget(&self) -> Self::Root {
        self.layout_box.clone()
    }

    fn view(&mut self, model: &ServerLayoutModel, _sender: Sender<<ServerLayoutModel as Model>::Msg>) {
        self.server_not_running_text.set_visible(!model.is_server_running);
    }
}
