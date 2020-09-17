#![deny(trivial_numeric_casts, trivial_casts, unsafe_code)]
#![warn(
    missing_crate_level_docs,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications
)]

//! Pixel drawing game for programmers inspired by reddits r/place

extern crate derive_more;

mod actor_framework;
pub mod canvas;
//mod net;

pub async fn start_server(_start_tcp: bool, _start_udp: bool) {
    /*
    let tcp_server = net::tcp_server::TcpServer::start_default();
    let canvas_server = canvas::LocalCanvas::start_default();

    let set_result = canvas_server
        .send(canvas::messages::SetPixelMsg::new(0, 0, [42, 24, 42]))
        .await;
    let get_result = canvas_server
        .send(canvas::messages::GetPixelMsg::new(0, 0))
        .await;
    let size_result = canvas_server
        .send(canvas::messages::GetSizeMsg::new())
        .await;

    println!("{:?}, {:?}, {:?}", set_result, get_result, size_result)

     */
}

pub async fn connect_to_server() {}
