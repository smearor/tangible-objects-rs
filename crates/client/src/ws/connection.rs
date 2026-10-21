//! `TungsteniteConnection` - the `tokio-tungstenite` socket adapted
//! to `graphql-ws-client`'s [`Connection`] trait; frame and error
//! mapping live here.

use futures::FutureExt;
use futures::SinkExt;
use futures_lite::stream::StreamExt;
use graphql_ws_client::Connection;
use graphql_ws_client::Error;
use graphql_ws_client::Message;
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;

use crate::ClientError;
use crate::Endpoint;
use crate::ws::message;

/// `tokio-tungstenite` socket adapted to `graphql-ws-client`'s
/// [`Connection`] trait - message and error mapping live here.
pub(crate) struct TungsteniteConnection {
    /// The connected `graphql-transport-ws` socket.
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl TungsteniteConnection {
    /// Opens the `graphql-transport-ws` socket to `endpoint`.
    pub(crate) async fn connect(endpoint: &Endpoint) -> Result<Self, ClientError> {
        let mut request = endpoint.ws_url().into_client_request()?;
        request.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            HeaderValue::from_static("graphql-transport-ws"),
        );
        let (socket, _) = connect_async(request).await?;
        Ok(Self { socket })
    }
}

impl Connection for TungsteniteConnection {
    /// Converts inbound tungstenite frames into
    /// `graphql-ws-client` [`Message`]s; transport errors terminate
    /// the stream.
    fn receive(&mut self) -> impl Future<Output = Option<Message>> + Send {
        self.socket
            .next()
            .map(|item| item.and_then(Result::ok).map(message::to_protocol))
            .map(Option::flatten)
    }

    /// Converts outbound [`Message`]s into tungstenite frames.
    fn send(&mut self, message: Message) -> impl Future<Output = Result<(), Error>> + Send {
        async move {
            self.socket
                .send(message::into_frame(message))
                .await
                .map_err(|error| Error::Send(error.to_string()))
        }
    }
}
