use gtk::glib::Sender;
use gtk::prelude::*;
use relm4::{send, ComponentUpdate, Model, Widgets};

use crate::gui::config_form::ConfigFormMsg;

use super::config_form::ConfigFormModel;

/// State of the *ControlButtons* component.
///
/// This component's job is to display multiple buttons which are used to start and stop the server or client.
pub(super) struct ControlButtonsModel {
    enabled: bool,
    server_currently_running: bool,
}

pub(super) enum ControlButtonsMsg {
    SetEnabled(bool),
    ToggleServerCurrentlyRunning,
}

impl Model for ControlButtonsModel {
    type Msg = ControlButtonsMsg;
    type Widgets = ControlButtonsWidgets;
    type Components = ();
}

impl ComponentUpdate<ConfigFormModel> for ControlButtonsModel {
    fn init_model(_parent_model: &ConfigFormModel) -> Self {
        Self {
            enabled: true,
            server_currently_running: false,
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        _sender: Sender<Self::Msg>,
        parent_sender: Sender<<ConfigFormModel as Model>::Msg>,
    ) {
        match msg {
            ControlButtonsMsg::ToggleServerCurrentlyRunning => {
                log::debug!("Toggling whether server is currently running");
                self.server_currently_running = !self.server_currently_running;
                send!(
                    parent_sender,
                    if self.server_currently_running {
                        ConfigFormMsg::SendStartServer
                    } else {
                        ConfigFormMsg::SendStopServer
                    }
                );
            }
            ControlButtonsMsg::SetEnabled(value) => self.enabled = value,
        }
    }
}

/// Widgets that are used to render [`ControlButtonsModel`]
pub(super) struct ControlButtonsWidgets {
    start_stop_button: gtk::Button,
}

impl Widgets<ControlButtonsModel, ConfigFormModel> for ControlButtonsWidgets {
    type Root = gtk::Button;

    fn init_view(
        _model: &ControlButtonsModel,
        _components: &<ControlButtonsModel as Model>::Components,
        sender: Sender<<ControlButtonsModel as Model>::Msg>,
    ) -> Self {
        let start_stop_button = gtk::Button::builder()
            .name("PixelflutControlButtonsButton")
            .label("Start Server")
            .build();
        start_stop_button
            .connect_clicked(move |_| send!(sender, ControlButtonsMsg::ToggleServerCurrentlyRunning));

        Self { start_stop_button }
    }

    fn root_widget(&self) -> Self::Root {
        self.start_stop_button.clone()
    }

    fn view(&mut self, model: &ControlButtonsModel, _sender: Sender<<ControlButtonsModel as Model>::Msg>) {
        self.start_stop_button.set_sensitive(model.enabled);

        if model.server_currently_running {
            self.start_stop_button.set_label("Stop Server");
        } else {
            self.start_stop_button.set_label("Start Server");
        }
    }
}
