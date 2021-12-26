use std::process::exit;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

pub fn start_gui<S>(args: &[S]) where S: AsRef<str> {
    let app = Application::builder()
        .application_id("me.finn-thorben.pixelflut")
        .build();

    app.connect_activate(|app| {
        // create the main window
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .title("Pixelflut")
            .build();

        // show the main window
        window.show();
    });

    // run main event loop and pass through its exit code
    exit(app.run_with_args(args))
}
