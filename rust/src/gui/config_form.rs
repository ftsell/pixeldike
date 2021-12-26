use gtk::glib::Sender;
use gtk::prelude::*;
use relm4::{Components, ComponentUpdate, Model, RelmComponent, WidgetPlus, Widgets};
use super::app::AppModel;
use super::layout::LayoutModel;
use super::control_buttons::ControlButtonsModel;


/// State of the *ConfigForm* component
///
/// The *ConfigForm* is rendered at the top of the main window and allows the user to configure
/// the application.
pub(super) struct ConfigFormModel {}

/// Operations which can change [`ConfigFormModel`]
pub(super) enum ConfigFormMsg {}

impl Model for ConfigFormModel {
    type Msg = ConfigFormMsg;
    type Widgets = ConfigFormWidgets;
    type Components = ConfigFormComponents;
}

impl ComponentUpdate<LayoutModel> for ConfigFormModel {
    fn init_model(_parent_model: &LayoutModel) -> Self {
        Self {}
    }

    fn update(&mut self, msg: Self::Msg, components: &Self::Components, sender: Sender<Self::Msg>, parent_sender: Sender<<LayoutModel as Model>::Msg>) {    }
}


/// Storage of instantiated gtk widgets that render [`ConfigFormModel`]
pub(super) struct ConfigFormWidgets {
    container: gtk::Box,
    spacer: gtk::Box,
    text: gtk::Text,
}

impl Widgets<ConfigFormModel, LayoutModel> for ConfigFormWidgets {
    type Root = gtk::Box;

    fn init_view(_model: &ConfigFormModel, components: &<ConfigFormModel as Model>::Components, _sender: Sender<<ConfigFormModel as Model>::Msg>) -> Self {
        let container = gtk::Box::builder()
            .name("PixelflutConfigFormContainer")
            .halign(gtk::Align::Fill)
            .orientation(gtk::Orientation::Horizontal)
            .build();
        container.set_margin_all(4);

        let text = gtk::Text::builder()
            .name("PixelflutConfigFormTestText")
            .text("Test text")
            .build();

        let spacer = gtk::Box::builder()
            .name("PixelflutConfigFormSpacer")
            .hexpand(true)
            .build();

        container.append(&text);
        container.append(&spacer);
        container.append(components.control_buttons.root_widget());

        Self {
            container,
            spacer,
            text,
        }
    }

    fn root_widget(&self) -> Self::Root {
        self.container.clone()
    }

    fn view(&mut self, _model: &ConfigFormModel, _sender: Sender<<ConfigFormModel as Model>::Msg>) {
    }
}


/// Child-Components of the *ConfigForm* components
pub(super) struct ConfigFormComponents {
    control_buttons: RelmComponent<ControlButtonsModel, ConfigFormModel>,
}

impl Components<ConfigFormModel> for ConfigFormComponents {
    fn init_components(parent_model: &ConfigFormModel, parent_sender: Sender<<ConfigFormModel as Model>::Msg>) -> Self {
        Self {
            control_buttons: RelmComponent::new(parent_model, parent_sender),
        }
    }

    fn connect_parent(&mut self, parent_widgets: &<ConfigFormModel as Model>::Widgets) {
        self.control_buttons.connect_parent(parent_widgets);
    }
}
