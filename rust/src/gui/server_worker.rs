use gtk::glib::Sender;
use relm4::{ComponentUpdate, Model};
use tokio::task::JoinHandle;

use pixelflut::pixmap::InMemoryPixmap;
use pixelflut::state_encoding::SharedMultiEncodings;

use super::app::AppModel;
use super::config_form::ProtocolChoice;

/// State of the *ServerWorker* component.
///
/// The *ServerWorker* is where the interaction with the rest of the program takes place for
/// server contexts.
/// The GUI communicates with this worker via [`ServerWorkerMsg`] and the *ServerWorker* sends
/// [`AppMsg`] back to the application.
pub(super) struct ServerWorkerModel {
    running_server: Option<PixelflutServer>,
}

struct PixelflutServer {
    pixmap: InMemoryPixmap,
    encodings: SharedMultiEncodings,
    join_handles: Vec<JoinHandle<()>>,
}

pub(super) enum ServerWorkerMsg {
    /// Start the server with the specified parameters
    StartServer { protocol: ProtocolChoice, port: u32 },
    /// Stop the server if it is running, ignore if not
    StopServer,
}

impl Model for ServerWorkerModel {
    type Msg = ServerWorkerMsg;
    type Widgets = ();
    type Components = ();
}

impl ComponentUpdate<AppModel> for ServerWorkerModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        Self { running_server: None }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        _sender: Sender<Self::Msg>,
        _parent_sender: Sender<<AppModel as Model>::Msg>,
    ) {
        match msg {
            ServerWorkerMsg::StartServer { .. } => {
                todo!("Starting a server is not yet implemented")
            }
            ServerWorkerMsg::StopServer => todo!("Stopping a server is not yet implemented"),
        }
    }
}
