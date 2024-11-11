use shared::{backend::Backend, frontend::FrontendClient};
use tarpc::context;

#[derive(Debug, Clone)]
pub struct BackendServer {
    player_client: FrontendClient,
}

impl Backend for BackendServer {
    async fn authenticate(self, ctx: context::Context, name: String) {
        println!("Authenticate: {}", name);
        // TODO: Do not unwrap here - client can disconnect mid request...
        self.player_client.event(ctx).await.unwrap();
    }
}

impl BackendServer {
    pub fn new(player_client: FrontendClient) -> BackendServer {
        BackendServer { player_client }
    }
}
