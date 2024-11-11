#![feature(extract_if)]
#![feature(async_closure)]

use std::{
    convert::identity,
    env, future,
    net::{Ipv4Addr, SocketAddrV4},
};

use any_spawner::Executor;
use futures::StreamExt;
use shared::{backend::Backend, frontend::FrontendClient, spawn_two_way::spawn_two_way};
use tarpc::{client, server::Channel};

use crate::server::BackendServer;

mod server;
mod ws;

static DEFAULT_SERVER_PORT: &str = "3600";

#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    Executor::init_tokio().unwrap();

    let server_address = SocketAddrV4::new(
        Ipv4Addr::LOCALHOST,
        env::var("SERVER_PORT")
            .unwrap_or(DEFAULT_SERVER_PORT.into())
            .parse()
            .expect("$SERVER_PORT should be a valid port"),
    );

    println!("Starting server on {}", server_address);

    ws::bind(server_address)
        .await
        .filter_map(|r| future::ready(r.ok()))
        .map(spawn_two_way)
        .map(|(server_transport, client_transport)| {
            let player_client =
                FrontendClient::new(client::Config::default(), client_transport).spawn();
            tarpc::server::BaseChannel::with_defaults(server_transport)
                .execute(BackendServer::new(player_client).serve())
                .for_each_concurrent(None, identity)
        })
        .for_each_concurrent(None, identity)
        .await;
}
