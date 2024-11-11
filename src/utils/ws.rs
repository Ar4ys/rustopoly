use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use ws_stream_wasm::WsMeta;

pub async fn bind<Item, SinkItem>(
    address: &str,
) -> Result<impl tarpc::Transport<SinkItem, Item, TransportError = std::io::Error>, std::io::Error>
where
    Item: for<'de> Deserialize<'de>,
    SinkItem: Serialize,
{
    tracing::info!("Connecting to server: {address}");

    WsMeta::connect(address, None)
        .map_ok(|(_, wsio)| {
            let frame = Framed::new(wsio.into_io(), LengthDelimitedCodec::new());
            tarpc::serde_transport::new(
                frame,
                tokio_serde::formats::Json::<Item, SinkItem>::default(),
            )
        })
        .map_err(|e| {
            tracing::error!("Errored on WsMeta connect\n{:?}", e);
            std::io::Error::from(std::io::ErrorKind::ConnectionRefused)
        })
        .await
}
