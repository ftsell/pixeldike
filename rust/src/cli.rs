use clap::{crate_description, crate_name, crate_version, App, Arg, ArgGroup, SubCommand};

/// Adapted implementation of [`clap::value_t_or_exit`] that ignores non-existing values and simply
/// keeps them as None without raising an error
#[macro_export]
macro_rules! value_t_or_exit_opt {
    ($m:ident, $v:expr, $t:ty) => {
        value_t_or_exit_opt!($m.value_of($v), $t)
    };
    ($m:ident.value_of($v:expr), $t:ty) => {
        $m.value_of($v).map(|v| match v.parse::<$t>() {
            Ok(val) => val,
            Err(_) => {
                clap::Error::value_validation_auto(format!("The argument '{}' isn't a valid value", v)).exit()
            }
        })
    };
}

pub fn get_app() -> App<'static, 'static> {
    let mut app = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .subcommand(
            SubCommand::with_name("server")
                .about("start a pixelflut server with the specified listeners")
                .arg(
                    Arg::with_name("tcp_port")
                        .long("tcp")
                        .takes_value(true)
                        .help("port on which to start a tcp listener"),
                )
                .arg(
                    Arg::with_name("udp_port")
                        .long("udp")
                        .takes_value(true)
                        .help("port on which to start a udp listener"),
                )
                .arg(
                    Arg::with_name("ws_port")
                        .long("ws")
                        .takes_value(true)
                        .help("port on which to start a websocket listener"),
                )
                .arg(
                    Arg::with_name("width")
                        .long("width")
                        .help("width of the pixmap on this server")
                        .takes_value(true)
                        .default_value("800"),
                )
                .arg(
                    Arg::with_name("height")
                        .long("height")
                        .help("height of the pixmap on this server")
                        .takes_value(true)
                        .default_value("600"),
                )
                .arg(
                    Arg::with_name("path")
                        .long("path")
                        .short("p")
                        .help("File path into which the pixmap is persisted")
                        .required(true)
                        .takes_value(true),
                )
                .group(
                    ArgGroup::with_name("listeners")
                        .args(&["tcp_port", "udp_port", "ws_port"])
                        .required(true)
                        .multiple(true),
                ),
        );

    if cfg!(feature = "gui") {
        app = app.subcommand(
            SubCommand::with_name("gui")
                .about("Start a graphical user interface")
        );
    }

    app
}
