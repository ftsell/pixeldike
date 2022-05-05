//! The *ServerWorker* is where the interaction with the rest of the program takes place for
//! server contexts.
//! The GUI communicates with this worker via [`ServerWorkerMsg`] and the *ServerWorker* sends
//! [`ServerHolderMsg`] back to the application.

use gtk::glib::Sender;
use pixelflut::pixmap::{Color, Pixmap};
use relm4::{ComponentUpdate, Model};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

use pixelflut::state_encoding::SharedMultiEncodings;

use crate::gui::components::server::server_config_form::ProtocolChoice;
use crate::gui::components::server::server_holder::ServerHolderModel;

type ParentModel = ServerHolderModel;

/// State of the *ServerWorker* component.
pub(in crate::gui) struct ServerWorkerModel {
    running_server: Option<PixelflutServer>,
}

struct PixelflutServer {
    encodings: SharedMultiEncodings,
    join_handles: Vec<JoinHandle<()>>,
}

pub(in crate::gui) enum ServerWorkerMsg {
    /// Start the server with the specified parameters
    StartServer {
        protocol: ProtocolChoice,
        port: u32,
        pixmap: force_send_sync::SendSync<gtk::gdk_pixbuf::Pixbuf>,
    },
    /// Stop the server if it is running, ignore if not
    StopServer,
}

impl Model for ServerWorkerModel {
    type Msg = ServerWorkerMsg;
    type Widgets = ();
    type Components = ();
}

impl ComponentUpdate<ParentModel> for ServerWorkerModel {
    fn init_model(_parent_model: &ParentModel) -> Self {
        Self { running_server: None }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        _sender: Sender<Self::Msg>,
        _parent_sender: Sender<<ParentModel as Model>::Msg>,
    ) {
        match msg {
            ServerWorkerMsg::StartServer { pixmap, .. } => {
                log::debug!("Starting server");
                match pixmap.put_raw_data(&vec![
                    Color(255, 0, 0);
                    (pixmap.width() * pixmap.height()) as usize
                ]) {
                    Err(e) => log::warn!("Could not set pixbuf to red: {}", e),
                    Ok(_) => {}
                }
                // TODO Actually start the server
            }
            ServerWorkerMsg::StopServer => {
                log::debug!("Stopping server");
                self.running_server = None;
                // TODO perform a clean shutdown
            }
        }
    }
}
