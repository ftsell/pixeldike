use gtk::glib::Sender;
use gtk::prelude::*;
use relm4::{AppUpdate, Model, RelmComponent, RelmWorker, Widgets};

use crate::gui::components::layout::LayoutModel;
use crate::gui::server_worker::{ServerWorkerModel, ServerWorkerMsg};

/// All data that is general to the whole application
pub(super) struct AppModel {}

/// Operations which can change [`AppModel`] data
pub(super) enum AppMsg {
    PropagateServerWorkerMsg(ServerWorkerMsg),
}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(&mut self, msg: Self::Msg, components: &Self::Components, _sender: Sender<Self::Msg>) -> bool {
        match msg {
            AppMsg::PropagateServerWorkerMsg(msg) => components
                .server_worker
                .send(msg)
                .expect("Could not send message to ServerWorker"),
        };

        true
    }
}

/// Memory location for instantiated gtk widgets
pub(super) struct AppWidgets {
    window: gtk::ApplicationWindow,
}

impl Widgets<AppModel, ()> for AppWidgets {
    type Root = gtk::ApplicationWindow;

    fn init_view(
        _model: &AppModel,
        components: &<AppModel as Model>::Components,
        _sender: Sender<<AppModel as Model>::Msg>,
    ) -> Self {
        let app_window = gtk::ApplicationWindow::builder()
            .title("Pixelflut")
            .default_width(800)
            .default_height(800)
            .build();

        app_window.set_child(Some(components.layout.root_widget()));

        Self { window: app_window }
    }

    fn root_widget(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(&mut self, _model: &AppModel, _sender: Sender<AppMsg>) {}
}

/// Storage struct for instantiated relm components
#[derive(relm4_macros::Components)]
pub(super) struct AppComponents {
    layout: RelmComponent<LayoutModel, AppModel>,
    server_worker: RelmWorker<ServerWorkerModel, AppModel>,
}
