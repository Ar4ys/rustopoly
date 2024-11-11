use std::{ops::Deref, sync::Arc};

use any_spawner::Executor;
use futures::{stream::AbortHandle, TryFutureExt};
use tarpc::{client, ClientMessage, Response, Transport};

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
    _server_handle: Arc<FrontendServerHandle>,
}

impl Deref for BackendClient {
    type Target = private::BackendClient;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl BackendClient {
    pub fn new(
        transport: impl Transport<ClientMessage<BackendRequest>, Response<BackendResponse>>
            + Send
            + 'static,
        server_handle: AbortHandle,
    ) -> Self {
        let client = private::BackendClient::new(client::Config::default(), transport);

        Executor::spawn(
            client
                .dispatch
                .unwrap_or_else(|e| tracing::error!("Connection broken: {}", e)),
        );

        Self {
            client: client.client,
            _server_handle: Arc::new(FrontendServerHandle(server_handle)),
        }
    }
}

#[derive(Debug, Clone)]
struct FrontendServerHandle(AbortHandle);

impl Drop for FrontendServerHandle {
    fn drop(&mut self) {
        self.0.abort();
    }
}
