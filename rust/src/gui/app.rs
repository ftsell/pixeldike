use crate::gui::components::server::server_holder::ServerHolderModel;
use gtk::glib::Sender;
use gtk::prelude::*;
use relm4::{AppUpdate, Model, RelmComponent, Widgets};

/// All data that is general to the whole application
pub(super) struct AppModel {}

/// Operations which can change [`AppModel`] data
pub(super) enum AppMsg {}

impl Model for AppModel {
    type Msg = AppMsg;
    type Widgets = AppWidgets;
    type Components = AppComponents;
}

impl AppUpdate for AppModel {
    fn update(
        &mut self,
        _msg: Self::Msg,
        _components: &Self::Components,
        _sender: Sender<Self::Msg>,
    ) -> bool {
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

        app_window.set_child(Some(components.server_holder.root_widget()));

        Self { window: app_window }
    }

    fn root_widget(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(&mut self, _model: &AppModel, _sender: Sender<AppMsg>) {}
}

/// Storage struct for instantiated relm components
#[derive(relm4::Components)]
pub(super) struct AppComponents {
    server_holder: RelmComponent<ServerHolderModel, AppModel>,
    // server_worker: RelmWorker<ServerWorkerModel, AppModel>,
}
