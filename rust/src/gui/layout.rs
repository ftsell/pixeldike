use gtk::glib::Sender;
use gtk::prelude::*;
use relm4::{Model, ComponentUpdate, Widgets, Components, WidgetPlus, RelmComponent};
use relm4::factory::FactoryView;
use super::app::{AppModel, AppMsg, AppWidgets};
use super::config_form::{ConfigFormModel, ConfigFormWidgets};

/// Representation of the layout state
pub(super) struct LayoutModel {}

impl Model for LayoutModel {
    type Msg = ();
    type Widgets = LayoutWidgets;
    type Components = LayoutComponents;
}

impl ComponentUpdate<AppModel> for LayoutModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        Self {}
    }

    fn update(&mut self, _msg: Self::Msg, _components: &Self::Components, _sender: Sender<Self::Msg>, _parent_sender: Sender<AppMsg>) {}
}


/// Storage of instantiated widgets
pub(super) struct LayoutWidgets {
    layout_box: gtk::Box,
    separator: gtk::Separator,
    text2: gtk::Text,
}

impl Widgets<LayoutModel, AppModel> for LayoutWidgets {
    type Root = gtk::Box;

    fn init_view(_model: &LayoutModel, components: &<LayoutModel as Model>::Components, _sender: Sender<<LayoutModel as Model>::Msg>) -> Self {
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

    fn view(&mut self, model: &LayoutModel, sender: Sender<<LayoutModel as Model>::Msg>) {
        todo!()
    }
}


/// Storage of instantiated child components
pub(super) struct LayoutComponents {
    config_form: RelmComponent<ConfigFormModel, LayoutModel>,
}

impl Components<LayoutModel> for LayoutComponents {
    fn init_components(parent_model: &LayoutModel, parent_sender: Sender<<LayoutModel as Model>::Msg>) -> Self {
        Self {
            config_form: RelmComponent::new(parent_model, parent_sender),
        }
    }

    fn connect_parent(&mut self, parent_widgets: &<LayoutModel as Model>::Widgets) {
        self.config_form.connect_parent(parent_widgets);
    }
}
