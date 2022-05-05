//! ServerHolder is a component that fully manages all server related functionality
//! including rendering as well as networking

use crate::gui::app::AppModel;
use crate::gui::components::server::server_config_form::ProtocolChoice;
use crate::gui::components::server::server_layout::{ServerLayoutModel, ServerLayoutMsg};
use crate::gui::components::server::server_worker::{ServerWorkerModel, ServerWorkerMsg};
use gtk::glib::Sender;
use gtk::traits::BoxExt;
use pixelflut::pixmap::gdk_pixbuf_pixmap::default_gdk_pixbuf_pixmap;
use relm4::{send, ComponentUpdate, Components, Model, RelmComponent, RelmWorker, Widgets};

type ParentModel = AppModel;

/// State of the *ServerHolder* component
pub(crate) struct ServerHolderModel {
    pixbuf: Option<gtk::gdk_pixbuf::Pixbuf>,
}

/// State altering messages of the *ServerHolder* component
pub(crate) enum ServerHolderMsg {
    StartServer { protocol: ProtocolChoice, port: u32 },
    StopServer,
}

/// Sub-Components used by the *ServerHolder* component
#[derive(Components)]
pub(crate) struct ServerHolderComponents {
    layout: RelmComponent<ServerLayoutModel, ServerHolderModel>,
    server_worker: RelmWorker<ServerWorkerModel, ServerHolderModel>,
}

/// GTK widgets that are directly used to render the *ServerHolder* component
pub(crate) struct ServerHolderWidgets {
    root: gtk::Box,
}

impl Model for ServerHolderModel {
    type Msg = ServerHolderMsg;
    type Widgets = ServerHolderWidgets;
    type Components = ServerHolderComponents;
}

impl ComponentUpdate<ParentModel> for ServerHolderModel {
    fn init_model(_parent_model: &ParentModel) -> Self {
        Self { pixbuf: None }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        _sender: Sender<Self::Msg>,
        _parent_sender: Sender<<ParentModel as Model>::Msg>,
    ) {
        match msg {
            ServerHolderMsg::StartServer { protocol, port } => {
                let pixbuf = default_gdk_pixbuf_pixmap();
                self.pixbuf = Some(pixbuf.clone());
                send!(
                    components.server_worker,
                    ServerWorkerMsg::StartServer {
                        protocol,
                        port,
                        pixmap: unsafe { force_send_sync::SendSync::new(pixbuf) }
                    }
                );
            }
            ServerHolderMsg::StopServer => {
                send!(components.server_worker, ServerWorkerMsg::StopServer);
                self.pixbuf = None;
            }
        }

        send!(components.layout, ServerLayoutMsg::SetPixbuf(self.pixbuf.clone()));
    }
}

impl Widgets<ServerHolderModel, ParentModel> for ServerHolderWidgets {
    type Root = gtk::Box;

    fn init_view(
        _model: &ServerHolderModel,
        components: &<ServerHolderModel as Model>::Components,
        _sender: Sender<ServerHolderMsg>,
    ) -> Self {
        let root = gtk::Box::default();
        root.append(components.layout.root_widget());

        Self { root }
    }

    fn root_widget(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(&mut self, _model: &ServerHolderModel, _sender: Sender<<ServerHolderModel as Model>::Msg>) {
        // this component does not change its rendering based on model updates
    }
}
