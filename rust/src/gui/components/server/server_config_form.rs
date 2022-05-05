//! The *ConfigForm* is rendered at the top of the server GUI and allows the user to configure
//! server properties

use std::str::FromStr;

use gtk::glib::Sender;
use gtk::prelude::*;
use relm4::{send, ComponentUpdate, Components, Model, RelmComponent, WidgetPlus, Widgets};

use crate::gui::components::server::server_control_buttons::{ControlButtonsModel, ControlButtonsMsg};
use crate::gui::components::server::server_layout::ServerLayoutModel;
use crate::gui::components::server::server_layout::ServerLayoutMsg;

/// Available pixelflut network protocols that can be chosen in the GUI
#[derive(Debug, Copy, Clone)]
pub(crate) enum ProtocolChoice {
    TCP,
    UDP,
}

/// State of the *ConfigForm* component
pub(in crate::gui) struct ConfigFormModel {
    /// Whether user input is currently frozen (not possible) or enabled.
    ///
    /// The is usually frozen when a server is currently running in which case it is not possible
    /// to change any server configuration.
    is_input_frozen: bool,
    selected_protocol: Option<ProtocolChoice>,
    selected_port: Option<u32>,
}

/// Operations which can change [`ConfigFormModel`]
pub(in crate::gui) enum ConfigFormMsg {
    /// Set the value of [`ConfigFormModel::selected_protocol`]
    SetSelectedProtocol(Option<ProtocolChoice>),
    /// Set the value of [`ConfigFormModel::selected_port`]
    SetSelectedPort(Option<u32>),
    /// Send a message based on the current input up the component hierarchy to start a pixelflut
    /// server.
    SendStartServer,
    /// Send a message up the component hierarchy to stop a pixelflut server
    SendStopServer,
}

impl ConfigFormModel {
    fn is_valid(&self) -> bool {
        self.selected_protocol.is_some() && self.selected_port.is_some()
    }
}

impl Model for ConfigFormModel {
    type Msg = ConfigFormMsg;
    type Widgets = ConfigFormWidgets;
    type Components = ConfigFormComponents;
}

impl ComponentUpdate<ServerLayoutModel> for ConfigFormModel {
    fn init_model(_parent_model: &ServerLayoutModel) -> Self {
        Self {
            is_input_frozen: false,
            selected_port: Some(9876),
            selected_protocol: Some(ProtocolChoice::TCP),
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        components: &Self::Components,
        _sender: Sender<Self::Msg>,
        parent_sender: Sender<<ServerLayoutModel as Model>::Msg>,
    ) {
        match msg {
            ConfigFormMsg::SetSelectedProtocol(protocol) => self.selected_protocol = protocol,
            ConfigFormMsg::SetSelectedPort(port) => self.selected_port = port,
            ConfigFormMsg::SendStartServer => {
                log::debug!("Freezing ConfigForm input and propagating StartServer message");
                self.is_input_frozen = true;
                send!(
                    parent_sender,
                    ServerLayoutMsg::StartServer {
                        protocol: self.selected_protocol.expect(
                            "Selected protocol is None but this message should not be received in that case"
                        ),
                        port: self.selected_port.expect(
                            "Selected port is None but this message should not be received in that case"
                        )
                    }
                );
            }
            ConfigFormMsg::SendStopServer => {
                self.is_input_frozen = false;
                send!(parent_sender, ServerLayoutMsg::StopServer);
            }
        }

        components
            .control_buttons
            .send(ControlButtonsMsg::SetEnabled(self.is_valid()))
            .expect("Could not send SetEnabled to control_buttons");
    }
}

/// Storage of instantiated gtk widgets that render [`ConfigFormModel`]
#[allow(dead_code)]
pub(in crate::gui) struct ConfigFormWidgets {
    container: gtk::Box,
    spacer: gtk::Box,
    protocol_selector_label: gtk::Label,
    protocol_selector: gtk::DropDown,
    port_input_label: gtk::Label,
    port_input: gtk::Entry,
}

impl Widgets<ConfigFormModel, ServerLayoutModel> for ConfigFormWidgets {
    type Root = gtk::Box;

    fn init_view(
        _model: &ConfigFormModel,
        components: &<ConfigFormModel as Model>::Components,
        sender: Sender<<ConfigFormModel as Model>::Msg>,
    ) -> Self {
        // container
        let container = gtk::Box::builder()
            .name("PixelflutConfigFormContainer")
            .halign(gtk::Align::Fill)
            .orientation(gtk::Orientation::Horizontal)
            .build();
        container.set_margin_all(4);
        let spacer = gtk::Box::builder()
            .name("PixelflutConfigFormSpacer")
            .hexpand(true)
            .build();

        // protocol selection
        let protocol_selector_label = gtk::Label::builder()
            .name("PixelflutConfigFormProtocolSelectorLabel")
            .label("Protocols")
            .margin_end(4)
            .build();
        let protocol_selector = gtk::DropDown::builder()
            .name("PixelflutConfigFormProtocolSelector")
            .margin_end(16)
            .model(&gtk::StringList::new(&["tcp", "udp"]))
            .build();
        let sender2 = sender.clone();
        protocol_selector.connect_selected_item_notify(move |dropdown| match dropdown.selected() {
            0 => send!(
                sender2,
                ConfigFormMsg::SetSelectedProtocol(Some(ProtocolChoice::TCP))
            ),
            1 => send!(
                sender2,
                ConfigFormMsg::SetSelectedProtocol(Some(ProtocolChoice::UDP))
            ),
            _ => send!(sender2, ConfigFormMsg::SetSelectedProtocol(None)),
        });

        // port input
        let port_input_label = gtk::Label::builder()
            .name("PixelflutConfigFormPortInputLabel")
            .label("Port")
            .margin_end(4)
            .build();
        let port_input = gtk::Entry::builder()
            .name("PixelflutConfigFormPortInput")
            .input_purpose(gtk::InputPurpose::Number)
            .max_width_chars(6)
            .text("9876")
            .build();
        port_input.connect_changed(move |entry| match u32::from_str(entry.text().as_str()) {
            Ok(port) => send!(sender, ConfigFormMsg::SetSelectedPort(Some(port))),
            Err(_) => send!(sender, ConfigFormMsg::SetSelectedPort(None)),
        });

        // view construction
        container.append(&protocol_selector_label);
        container.append(&protocol_selector);
        container.append(&port_input_label);
        container.append(&port_input);
        container.append(&spacer);
        container.append(components.control_buttons.root_widget());

        Self {
            container,
            spacer,
            protocol_selector_label,
            protocol_selector,
            port_input_label,
            port_input,
        }
    }

    fn root_widget(&self) -> Self::Root {
        self.container.clone()
    }

    fn view(&mut self, model: &ConfigFormModel, _sender: Sender<<ConfigFormModel as Model>::Msg>) {
        self.protocol_selector.set_sensitive(!model.is_input_frozen);
        self.port_input.set_sensitive(!model.is_input_frozen);
    }
}

/// Child-Components of the *ConfigForm* components
pub(in crate::gui) struct ConfigFormComponents {
    control_buttons: RelmComponent<ControlButtonsModel, ConfigFormModel>,
}

impl Components<ConfigFormModel> for ConfigFormComponents {
    fn init_components(
        parent_model: &ConfigFormModel,
        parent_sender: Sender<<ConfigFormModel as Model>::Msg>,
    ) -> Self {
        Self {
            control_buttons: RelmComponent::new(parent_model, parent_sender),
        }
    }

    fn connect_parent(&mut self, parent_widgets: &<ConfigFormModel as Model>::Widgets) {
        self.control_buttons.connect_parent(parent_widgets);
    }
}
