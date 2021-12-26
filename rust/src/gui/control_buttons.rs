use gtk::glib::Sender;
use gtk::prelude::*;
use relm4::{Widgets, ComponentUpdate, Model};
use super::config_form::ConfigFormModel;

/// State of the *ControlButtons* component.
///
/// This component's job is to display multiple buttons which are used to start and stop the server or client.
pub(super) struct ControlButtonsModel {}

impl Model for ControlButtonsModel {
    type Msg = ();
    type Widgets = ControlButtonsWidgets;
    type Components = ();
}

impl ComponentUpdate<ConfigFormModel> for ControlButtonsModel {
    fn init_model(parent_model: &ConfigFormModel) -> Self {
        Self {}
    }

    fn update(&mut self, msg: Self::Msg, components: &Self::Components, sender: Sender<Self::Msg>, parent_sender: Sender<<ConfigFormModel as Model>::Msg>) {
    }
}


/// Widgets that are used to render [`ControlButtonsModel`]
pub(super) struct ControlButtonsWidgets {
    start_stop_button: gtk::Button,
}

impl Widgets<ControlButtonsModel, ConfigFormModel> for ControlButtonsWidgets {
    type Root = gtk::Button;

    fn init_view(model: &ControlButtonsModel, _components: &<ControlButtonsModel as Model>::Components, sender: Sender<<ControlButtonsModel as Model>::Msg>) -> Self {
        let start_stop_button = gtk::Button::builder()
            .name("PixelflutControlButtonsButton")
            .label("Start Server")
            .build();

        Self {
            start_stop_button,
        }
    }

    fn root_widget(&self) -> Self::Root {
        self.start_stop_button.clone()
    }

    fn view(&mut self, model: &ControlButtonsModel, sender: Sender<<ControlButtonsModel as Model>::Msg>) {}
}
