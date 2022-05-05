//! The *ServerWorker* is where the interaction with the rest of the program takes place for
//! server contexts.
//! The GUI communicates with this worker via [`ServerWorkerMsg`] and the *ServerWorker* sends
//! [`ServerHolderMsg`] back to the application.

use gtk::glib::Sender;
use pixelflut::net;
use pixelflut::pixmap::InMemoryPixmap;
use relm4::{send, ComponentUpdate, Model};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;
use tokio::{runtime, time};

use pixelflut::state_encoding::SharedMultiEncodings;

use crate::gui::components::server::server_config_form::ProtocolChoice;
use crate::gui::components::server::server_holder::{ServerHolderModel, ServerHolderMsg};
use crate::pixelflut::pixmap::Pixmap;

type ParentModel = ServerHolderModel;

/// State of the *ServerWorker* component.
pub(in crate::gui) struct ServerWorkerModel {
    runtime: Runtime,
    running_server: Option<PixelflutServer>,
}

struct PixelflutServer {
    encodings: SharedMultiEncodings,
    pixmap: Arc<InMemoryPixmap>,
    join_handles: Vec<JoinHandle<()>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(in crate::gui) enum ServerWorkerMsg {
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

impl ComponentUpdate<ParentModel> for ServerWorkerModel {
    fn init_model(_parent_model: &ParentModel) -> Self {
        let runtime = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Could not construct tokio runtime");

        Self {
            running_server: None,
            runtime,
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        _sender: Sender<Self::Msg>,
        parent_sender: Sender<<ParentModel as Model>::Msg>,
    ) {
        log::debug!("ComponentsUpdate: {:?}", msg);

        match msg {
            ServerWorkerMsg::StartServer { protocol, port } => {
                // create data structures
                let pixmap = Arc::new(InMemoryPixmap::default());
                let encodings = SharedMultiEncodings::default();

                // spawn server task
                let pixmap2 = pixmap.clone();
                let encodings2 = encodings.clone();
                let server_handle = self.runtime.spawn(async move {
                    run_server(protocol, port as u16, pixmap2, encodings2).await;
                });

                // spawn synchronizer task
                let pixmap2 = pixmap.clone();
                let parent_sender2 = parent_sender.clone();
                let synchronizer_handle = self.runtime.spawn(async move {
                    run_synchronizer(pixmap2, parent_sender2).await;
                });

                // set running server on self
                self.running_server = Some(PixelflutServer {
                    join_handles: vec![server_handle, synchronizer_handle],
                    pixmap,
                    encodings,
                });
            }
            ServerWorkerMsg::StopServer => {
                self.running_server = None;
                // TODO perform a clean shutdown
            }
        }
    }
}

async fn run_synchronizer(pixmap: Arc<InMemoryPixmap>, parent_sender: Sender<<ParentModel as Model>::Msg>) {
    const FPS: f32 = 5f32;
    let mut interval = time::interval(Duration::from_secs_f32(1f32 / FPS));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;
        send!(
            parent_sender,
            ServerHolderMsg::UpdatePixmap(pixmap.get_raw_data().expect("Could not get raw data"))
        )
    }
}

async fn run_server(
    protocol: ProtocolChoice,
    port: u16,
    pixmap: Arc<InMemoryPixmap>,
    encodings: SharedMultiEncodings,
) {
    match protocol {
        ProtocolChoice::TCP => {
            net::tcp_server::listen(
                pixmap,
                encodings,
                net::tcp_server::TcpOptions {
                    listen_address: SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port),
                },
            )
            .await
        }
        ProtocolChoice::UDP => {
            net::udp_server::listen(
                pixmap,
                encodings,
                net::udp_server::UdpOptions {
                    listen_address: SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port),
                },
            )
            .await
        }
    }
    .expect("Could not run server listener");
}
