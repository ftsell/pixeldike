use gtk::glib::Sender;
use gtk::prelude::*;
use relm4::{send, ComponentUpdate, Components, Model, RelmComponent, WidgetPlus, Widgets};

use crate::gui::app::{AppModel, AppMsg};
use crate::gui::components::config_form::ConfigFormModel;
use crate::gui::server_worker::ServerWorkerMsg;

/// Representation of the layout state
pub(in crate::gui) struct LayoutModel {}

pub(in crate::gui) enum LayoutMsg {
    PropagateServerWorkerMsg(ServerWorkerMsg),
}

impl Model for LayoutModel {
    type Msg = LayoutMsg;
    type Widgets = LayoutWidgets;
    type Components = LayoutComponents;
}

impl ComponentUpdate<AppModel> for LayoutModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        Self {}
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        _sender: Sender<Self::Msg>,
        parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            LayoutMsg::PropagateServerWorkerMsg(msg) => {
                send!(parent_sender, AppMsg::PropagateServerWorkerMsg(msg))
            }
        }
    }
}

/// Storage of instantiated widgets
#[allow(dead_code)]
pub(in crate::gui) struct LayoutWidgets {
    layout_box: gtk::Box,
    separator: gtk::Separator,
    text2: gtk::Text,
}

impl Widgets<LayoutModel, AppModel> for LayoutWidgets {
    type Root = gtk::Box;

    fn init_view(
        _model: &LayoutModel,
        components: &<LayoutModel as Model>::Components,
        _sender: Sender<<LayoutModel as Model>::Msg>,
    ) -> Self {
        let layout_box = gtk::Box::builder()
            .name("PixelflutLayout")
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .build();
        layout_box.set_margin_all(4);

        let separator = gtk::Separator::builder()
            .name("PixelflutLayoutSeparator")
            .orientation(gtk::Orientation::Vertical)
            .build();
        let text2 = gtk::Text::builder()
            .name("PixelflutLayoutPlaceholder")
            .text("There will be the pixel buffer here")
            .editable(false)
            .focusable(false)
            .max_width_chars(128)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();

        layout_box.append(components.config_form.root_widget());
        layout_box.append(&separator);
        layout_box.append(&text2);

        Self {
            layout_box,
            separator,
            text2,
        }
    }

    fn root_widget(&self) -> Self::Root {
        self.layout_box.clone()
    }

    fn view(&mut self, _model: &LayoutModel, _sender: Sender<<LayoutModel as Model>::Msg>) {}
}

/// Storage of instantiated child components
pub(in crate::gui) struct LayoutComponents {
    config_form: RelmComponent<ConfigFormModel, LayoutModel>,
}

impl Components<LayoutModel> for LayoutComponents {
    fn init_components(
        parent_model: &LayoutModel,
        parent_sender: Sender<<LayoutModel as Model>::Msg>,
    ) -> Self {
        Self {
            config_form: RelmComponent::new(parent_model, parent_sender),
        }
    }

    fn connect_parent(&mut self, parent_widgets: &<LayoutModel as Model>::Widgets) {
        self.config_form.connect_parent(parent_widgets);
    }
}
