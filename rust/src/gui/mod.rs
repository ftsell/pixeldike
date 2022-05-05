use libadwaita::Application;
use relm4::RelmApp;

mod app;
mod components;

pub fn start_gui<S>(args: &[S])
where
    S: AsRef<str>,
{
    gtk::init().expect("Couln't initialize GTK");
    let app = Application::builder()
        .application_id("me.finn-thorben.pixelflut")
        .build();
    let model = app::AppModel {};

    let app = RelmApp::with_app(model, app);
    app.run_with_args(args);

    /*

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
     */
}
