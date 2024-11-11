use any_spawner::Executor;
use futures::{io, Sink, SinkExt, Stream, StreamExt, TryFutureExt, TryStreamExt};
use tarpc::transport::channel::UnboundedChannel;

use std::error::Error;

/// A tarpc message that can be either a request or a response.
#[derive(serde::Serialize, serde::Deserialize)]
pub enum TwoWayMessage<Req, Resp> {
    ClientMessage(tarpc::ClientMessage<Req>),
    Response(tarpc::Response<Resp>),
}

pub type TwoWayTransports<Req1, Resp1, Req2, Resp2> = (
    UnboundedChannel<tarpc::ClientMessage<Req1>, tarpc::Response<Resp1>>,
    UnboundedChannel<tarpc::Response<Resp2>, tarpc::ClientMessage<Req2>>,
);

/// Returns two transports that multiplex over the given transport.
/// The first transport can be used by a server: it receives requests and sends back responses.
/// The second transport can be used by a client: it sends requests and receives back responses.
pub fn spawn_two_way<Req1, Resp1, Req2, Resp2, T>(
    transport: T,
) -> TwoWayTransports<Req1, Resp1, Req2, Resp2>
where
    T: Stream<Item = io::Result<TwoWayMessage<Req1, Resp2>>>,
    T: Sink<TwoWayMessage<Req2, Resp1>, Error = io::Error>,
    T: 'static,
    Req1: 'static,
    Resp1: 'static,
    Req2: 'static,
    Resp2: 'static,
{
    let (server, server_) = tarpc::transport::channel::unbounded();
    let (client, client_) = tarpc::transport::channel::unbounded();
    let (mut server_sink, server_stream) = server.split();
    let (mut client_sink, client_stream) = client.split();
    let (transport_sink, mut transport_stream) = transport.split();

    // Task for inbound message handling.
    Executor::spawn_local(async move {
        let e: Result<(), Box<dyn Error>> = async move {
            while let Some(msg) = transport_stream.next().await {
                match msg? {
                    TwoWayMessage::ClientMessage(req) => server_sink.send(req).await?,
                    TwoWayMessage::Response(resp) => client_sink.send(resp).await?,
                }
            }
            Ok(())
        }
        .await;
        match e {
            Ok(()) => eprintln!("Transport closed."),
            Err(e) => eprintln!("Failed to forward messages to server: {:?}", e),
        }
    });

    // Task for outbound message handling.
    Executor::spawn_local(
        futures::stream::select(
            server_stream.map_ok(|resp| TwoWayMessage::Response(resp)),
            client_stream.map_ok(|req| TwoWayMessage::ClientMessage(req)),
        )
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))
        .forward(transport_sink)
        .unwrap_or_else(|e| eprintln!("Failed to forward messages to transport: {:?}", e)),
    );
    (server_, client_)
}
