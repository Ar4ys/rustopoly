use async_stream::try_stream;
use futures::Stream;
use serde::{de::DeserializeOwned, Serialize};
use tokio::net::ToSocketAddrs;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use async_tungstenite::tokio::accept_async;
use ws_stream_tungstenite::*;

pub async fn bind<Item, SinkItem>(
    address: impl ToSocketAddrs,
) -> impl Stream<
    Item = Result<
        impl tarpc::Transport<SinkItem, Item, TransportError = std::io::Error>,
        std::io::Error,
    >,
>
where
    Item: DeserializeOwned,
    SinkItem: Serialize,
{
    tracing::info!("Binding RPC TCP Session");
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    tracing::info!("Bound, waiting on clients");

    try_stream! {
        loop {
            let (stream, addr) = listener.accept().await?;
            let ws = accept_async(stream).await.unwrap();
            let ws_stream = WsStream::new(ws);
            tracing::info!("New WebSocket connection: {}", addr);
            let frame = Framed::new(ws_stream, LengthDelimitedCodec::new());
            yield tarpc::serde_transport::new(
                frame,
                tokio_serde::formats::Json::<Item, SinkItem>::default(),
            );
        }
    }
}
