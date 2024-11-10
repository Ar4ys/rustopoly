#[tarpc::service]
pub trait Frontend {
    async fn event();
}
