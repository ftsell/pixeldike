//! ServerHolder is a component that fully manages all server related functionality
//! including rendering as well as networking

use crate::gui::app::AppModel;
use crate::gui::components::server::server_config_form::ProtocolChoice;
use crate::gui::components::server::server_layout::{ServerLayoutModel, ServerLayoutMsg};
use crate::gui::components::server::server_worker::{ServerWorkerModel, ServerWorkerMsg};
use gtk::glib::Sender;
use gtk::traits::BoxExt;
use pixelflut::pixmap::Color;
use relm4::{send, ComponentUpdate, Components, Model, RelmComponent, RelmWorker, Widgets};

type ParentModel = AppModel;

/// State of the *ServerHolder* component
pub(crate) struct ServerHolderModel {}

/// State altering messages of the *ServerHolder* component
pub(crate) enum ServerHolderMsg {
    StartServer { protocol: ProtocolChoice, port: u32 },
    StopServer,
    UpdatePixmapData(Box<Vec<Color>>),
}

/// Sub-Components used by the *ServerHolder* component
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
        Self {}
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
                send!(
                    components.server_worker,
                    ServerWorkerMsg::StartServer { protocol, port }
                );
            }
            ServerHolderMsg::StopServer => {
                send!(components.server_worker, ServerWorkerMsg::StopServer);
            }
            ServerHolderMsg::UpdatePixmapData(data) => {
                send!(components.layout, ServerLayoutMsg::UpdatePixmapData(data))
            }
        }
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

impl Components<ServerHolderModel> for ServerHolderComponents {
    fn init_components(parent_model: &ServerHolderModel, parent_sender: Sender<ServerHolderMsg>) -> Self {
        Self {
            layout: RelmComponent::new(parent_model, parent_sender.clone()),
            server_worker: RelmWorker::with_new_thread(parent_model, parent_sender),
        }
    }

    fn connect_parent(&mut self, parent_widgets: &ServerHolderWidgets) {
        self.layout.connect_parent(parent_widgets);
    }
}
