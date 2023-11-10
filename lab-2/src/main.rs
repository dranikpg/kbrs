#![feature(slice_pattern)]

use tokio;

mod client;
mod server;
mod common;

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    let arg = std::env::args().nth(1).unwrap_or("client".to_owned());

    if arg == "client" {
        rt.block_on(client::run())
    } else if arg == "server" {
        let server = server::Server {};
        rt.block_on(server.run())
    } else {
        panic!("Bad mode")
    }
}
