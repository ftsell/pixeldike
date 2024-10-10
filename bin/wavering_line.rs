use std::{collections::HashSet, io::Write, mem, ops::Add};

use bytes::{BufMut, BytesMut};
use clap::{ArgAction, Parser};
use pixeldike::{
    net::{
        clients::{TcpClient, UdpClient},
        protocol::{Request, Response},
    },
    pixmap::Color,
};
use rand::prelude::*;
use tokio::{io::AsyncWriteExt, task::LocalSet};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use url::Url;

const LINE_THICKNESS: u16 = 10;
const WALK_DISTANCE: u16 = 5;
const DIRECTION_SPREAD: i32 = 18;
const COLOR_EVOLUTION_RANGE: i8 = 2;

#[derive(Parser, Debug, Clone)]
struct Cli {
    /// Increase program verbosity
    ///
    /// The default verbosity level is INFO.
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count, default_value = "0")]
    pub verbose: u8,

    /// Decrease program verbosity
    ///
    /// The default verbosity level is INFO.
    #[arg(short = 'q', long = "quiet", action = ArgAction::Count, default_value = "0")]
    pub quiet: u8,

    /// Address of the pixelflut server
    #[arg(long = "data-url")]
    pub data_url: Url,

    /// Address to use for the management connection
    #[arg(long = "meta-url")]
    pub metadata_url: Url,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    init_logger(&args);

    // prepare async environment and run the client loop
    let local_set = LocalSet::new();
    local_set
        .run_until(async move {
            // fetch size from server via metadata url
            tracing::info!("Fetching metadata from pixelflut server");
            let (server_width, server_height) = match args.metadata_url.scheme() {
                "tcp" => {
                    let addr = args
                        .metadata_url
                        .socket_addrs(|| Some(1234))
                        .expect("Could not resolve metadata url address")[0];
                    tracing::debug!("Establishing metadata connection to {} ", addr);
                    let mut client = TcpClient::connect(&addr)
                        .await
                        .expect("Could not connect to server for metadata exchange");
                    tracing::debug!("Sending SIZE command");
                    client
                        .send_request(Request::GetSize)
                        .await
                        .expect("Could not send SIZE command to server");
                    client.flush().await.expect("Could not flush metadata connection");
                    tracing::debug!("Waiting for SIZE response");
                    match client
                        .await_response()
                        .await
                        .expect("Could not wait for SIZE response")
                    {
                        Response::Size { width, height } => {
                            tracing::debug!("Remote canvas has size {:?}", (width, height));
                            (width, height)
                        }
                        resp => panic!("Received unexpected response to SIZE command: {:?}", resp),
                    }
                }
                proto => panic!(
                    "Protocol {} is not supported for metadata connection. Use tcp instead.",
                    proto
                ),
            };

            // Connect to server for data exchange
            tracing::info!("Preparing socket for flooding to {}", args.data_url);
            let data_addr = args
                .data_url
                .socket_addrs(|| Some(1234))
                .expect("Could not resolve data url address")[0];
            tracing::debug!("Establishing data connection to {}", data_addr);
            let mut client = match args.data_url.scheme() {
                "tcp" => TcpClient::connect(&data_addr)
                    .await
                    .expect("Could not connect to server for data exchange"),
                proto => panic!(
                    "Protocol {} is not supported for data connection. Use tcp or udp instead",
                    proto
                ),
            };

            // generate an initial line
            let mut line = Line::make_initial(server_width as u16, server_height as u16);
            tracing::info!(
                "Line contains {} points and is color #{:x}",
                line.points.len(),
                line.color
            );

            // run client loop which continuously draws and evolves lines
            tracing::info!("Rendering wavering line");
            loop {
                line = line.evolve();
                let mut commands = HashSet::with_capacity(
                    line.points.len() * LINE_THICKNESS as usize * LINE_THICKNESS as usize,
                );
                let mut buf = BytesMut::with_capacity(commands.len() * mem::size_of::<Request>()).writer();

                for i_point in line.points.iter() {
                    for ix in i_point.0.saturating_sub(LINE_THICKNESS)..i_point.0 + LINE_THICKNESS {
                        for iy in i_point.1.saturating_sub(LINE_THICKNESS)..i_point.1 + LINE_THICKNESS {
                            if (ix as usize) < server_width && (iy as usize) < server_height {
                                commands.insert(Request::SetPixel {
                                    x: ix as usize,
                                    y: iy as usize,
                                    color: line.color,
                                });
                            }
                        }
                    }
                }

                for i_cmd in commands.iter() {
                    i_cmd.write(&mut buf).unwrap();
                }

                client
                    .get_writer()
                    .write_all(buf.get_ref())
                    .await
                    .expect("Could not write commands to server");
                client.flush().await.expect("Could not flush pixel commands");
            }
        })
        .await;
}

#[inline]
fn init_logger(args: &Cli) {
    // determine combined log level from cli arguments
    const DEFAULT_LEVEL: u8 = 3;
    let log_level = match DEFAULT_LEVEL
        .saturating_add(args.verbose)
        .saturating_sub(args.quiet)
    {
        0 => LevelFilter::OFF,
        1 => LevelFilter::ERROR,
        2 => LevelFilter::WARN,
        3 => LevelFilter::INFO,
        4 => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    };

    // configure appropriate level filter
    // tokio is very spammy on higher log levels which is usually not interesting so we filter it out
    let filter = tracing_subscriber::filter::Targets::new()
        .with_default(log_level)
        .with_target("tokio", Ord::min(LevelFilter::WARN, log_level))
        .with_target("runtime", Ord::min(LevelFilter::WARN, log_level));
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();
}

#[derive(Eq, PartialEq)]
struct Line {
    points: Vec<(u16, u16)>,
    canvas_size: (u16, u16),
    color: Color,
}

impl Line {
    /// Create an initial line that starts at a random position on the canvas edge and does a random walk
    fn make_initial(canvas_width: u16, canvas_height: u16) -> Self {
        let mut rng = rand::thread_rng();

        // generate an initial point on one of the sides
        let (initial_point, mut direction) = match rng.gen_range(0..4) {
            // top side
            0 => ((canvas_width / 2, 0), 180),
            // right side
            1 => ((canvas_width - 1, canvas_height / 2), 270),
            // bottom side
            2 => ((canvas_width / 2, canvas_height - 1), 0),
            // left side
            3 => ((0, canvas_height / 2), 90),
            _ => unreachable!(),
        };
        let mut points = vec![initial_point];
        tracing::info!(
            "Starting line at {:?} with direction {}°",
            initial_point,
            direction
        );

        // generate new points via random walk until another canvas edge is reached
        loop {
            let last_point = points.last().unwrap();
            direction =
                ((direction as i32 + rng.gen_range(-DIRECTION_SPREAD..DIRECTION_SPREAD)) % 360) as u16;

            // calculate the next point
            let d_x = ((direction as f32).to_radians().sin() * (WALK_DISTANCE as f32)).round() as i16;
            let d_y = ((direction as f32).to_radians().cos() * (WALK_DISTANCE as f32)).round() as i16;
            let next_point = (
                ((last_point.0 as i16) + d_x) as u16,
                ((last_point.1 as i16) - d_y) as u16,
            );

            // continue only if next_point is still in canvas bounds
            if next_point.0 > 0
                && next_point.0 < canvas_width
                && next_point.1 > 0
                && next_point.1 < canvas_height
            {
                points.push(next_point);
                continue;
            } else {
                break;
            }
        }

        Self {
            points,
            canvas_size: (canvas_width, canvas_height),
            color: Color::from((
                rand::thread_rng().gen_range(0..=255),
                rand::thread_rng().gen_range(0..=255),
                rand::thread_rng().gen_range(0..=255),
            )),
        }
    }

    fn evolve(self) -> Self {
        let mut rng = rand::thread_rng();

        // choose a random point on the line and split the line into the two parts separated by that point
        let _start_point_idx = rng.gen_range(0..self.points.len());
        let start_point_idx = 0;
        let (_points1, points2) = self.points.split_at(start_point_idx);

        // walk the line from the starting point, modifying each choice by a random amount
        let mut point_iter = points2.iter().peekable();
        while let Some(i_point) = point_iter.next() {
            let next_point = match point_iter.peek() {
                None => continue,
                Some(&n) => n,
            };

            todo!();
        }

        Self {
            points: self.points,
            canvas_size: self.canvas_size,
            color: self.color,
        }
    }
}
