use std::{ops::Deref, sync::Arc};

use tarpc::{client, ClientMessage, Response, Transport};
use tokio::task::JoinHandle;

mod private {
    #[tarpc::service]
    pub trait Backend {
        async fn authenticate(name: String);
    }
}

pub use private::{Backend, BackendRequest, BackendResponse, ServeBackend};

#[derive(Debug, Clone)]
pub struct BackendClient {
    client: private::BackendClient,
    server_handle: Arc<JoinHandle<()>>,
}

impl Deref for BackendClient {
    type Target = private::BackendClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl Drop for BackendClient {
    fn drop(&mut self) {
        self.server_handle.abort();
    }
}

impl BackendClient {
    pub fn new(
        transport: impl Transport<ClientMessage<BackendRequest>, Response<BackendResponse>>
            + Send
            + 'static,
        server_handle: JoinHandle<()>,
    ) -> Self {
        Self {
            client: private::BackendClient::new(client::Config::default(), transport).spawn(),
            server_handle: Arc::new(server_handle),
        }
    }
}
