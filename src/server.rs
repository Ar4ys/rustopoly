use std::convert::identity;

use futures::{
    stream::{AbortHandle, Abortable},
    FutureExt, StreamExt,
};
use leptos::spawn::spawn_local;
use shared::{backend::BackendClient, frontend::Frontend, spawn_two_way::spawn_two_way};
use tarpc::{context, server::Channel};

use crate::utils::ws;

#[derive(Clone)]
pub struct FrontendServer {}

impl FrontendServer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Frontend for FrontendServer {
    async fn event(self, _: context::Context) {}
}

pub async fn connect_to_server() -> Result<BackendClient, std::io::Error> {
    let transport = ws::bind("").await?;
    let (server_transport, client_transport) = spawn_two_way(transport);
    let (server_handle, registration) = AbortHandle::new_pair();
    let server_fut = tarpc::server::BaseChannel::with_defaults(server_transport)
        .execute(FrontendServer::new().serve())
        .for_each_concurrent(None, identity);

    spawn_local(Abortable::new(server_fut, registration).map(|_| {}));

    Ok(BackendClient::new(client_transport, server_handle))
}
