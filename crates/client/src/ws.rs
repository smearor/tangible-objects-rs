//! `graphql-transport-ws` subscription transport - a raw
//! `tokio-tungstenite` socket speaking the four-message handshake
//! (`connection_init`/`connection_ack`/`subscribe` → `next`/`complete`).

use futures::SinkExt;
use futures_core::Stream;
use futures_lite::StreamExt;
use serde_json::Value;
use serde_json::json;
use tangible_objects_model::TangibleEvent;
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;

use crate::ClientError;
use crate::Endpoint;

/// Raw `graphql-transport-ws` socket - wraps the tungstenite frame
/// plumbing behind JSON send/receive.
pub(crate) struct WebSocketClient {
    /// The connected socket (`connection_init`/`connection_ack` done).
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WebSocketClient {
    /// Connects to `endpoint`'s WebSocket URL (subprotocol
    /// `graphql-transport-ws`) and performs the
    /// `connection_init`/`connection_ack` handshake.
    async fn connect(endpoint: &Endpoint) -> Result<Self, ClientError> {
        let mut request = endpoint
            .ws_url()
            .into_client_request()
            .map_err(ClientError::Ws)?;
        request.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            HeaderValue::from_static("graphql-transport-ws"),
        );
        let (socket, _response) = connect_async(request).await?;
        let mut client = Self { socket };

        client.send(json!({"type": "connection_init"})).await?;
        loop {
            let message = client.next_message().await?;
            let ty = message
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default();
            match ty {
                "connection_ack" => break,
                "ping" | "pong" => continue,
                other => {
                    return Err(ClientError::Protocol(format!(
                        "expected connection_ack, got {other}"
                    )));
                }
            }
        }
        Ok(client)
    }

    /// Writes one JSON control message onto the socket.
    async fn send(&mut self, value: Value) -> Result<(), ClientError> {
        self.socket
            .send(Message::Text(value.to_string().into()))
            .await
            .map_err(ClientError::Ws)
    }

    /// Reads the next text frame as parsed JSON.
    async fn next_message(&mut self) -> Result<Value, ClientError> {
        loop {
            match self.socket.next().await {
                Some(Ok(Message::Text(text))) => {
                    return serde_json::from_str(&text)
                        .map_err(|e| ClientError::Protocol(e.to_string()));
                }
                Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => continue,
                Some(Ok(_)) => continue,
                Some(Err(e)) => return Err(ClientError::Ws(e)),
                None => return Err(ClientError::Protocol("socket closed".into())),
            }
        }
    }

    /// Sends the `subscribe` payload (`id` = `field`) and consumes the
    /// socket into the decoded `TangibleEvent` stream - ends on
    /// `complete`, socket close or the first protocol violation.
    pub(crate) async fn subscribe(
        mut self,
        field: &str,
        query: &str,
        variables: Value,
    ) -> Result<impl Stream<Item = Result<TangibleEvent, ClientError>> + 'static, ClientError> {
        self.send(json!({
            "id": field,
            "type": "subscribe",
            "payload": { "query": query, "variables": variables },
        }))
        .await?;
        Ok(self.into_event_stream(field))
    }

    /// Consumes the socket into the subscription's decoded
    /// `TangibleEvent` stream (`field` selects the response key).
    fn into_event_stream(
        self,
        field: &str,
    ) -> impl Stream<Item = Result<TangibleEvent, ClientError>> + 'static {
        let field = field.to_string();
        self.socket.filter_map(move |result| {
            let message = match result {
                Ok(message) => message,
                Err(e) => return Some(Err(ClientError::Ws(e))),
            };
            let text = match message {
                Message::Text(text) => text,
                _ => return None,
            };
            let frame: Value = match serde_json::from_str(&text) {
                Ok(frame) => frame,
                Err(e) => return Some(Err(ClientError::Protocol(e.to_string()))),
            };
            match frame.get("type").and_then(Value::as_str) {
                Some("next") => {
                    let payload = frame.pointer(&format!("/payload/data/{field}"));
                    match payload {
                        Some(value) => match serde_json::from_value(value.clone()) {
                            Ok(event) => Some(Ok(event)),
                            Err(e) => Some(Err(ClientError::Decode(e))),
                        },
                        None => Some(Err(ClientError::Protocol(
                            "next message without event payload".into(),
                        ))),
                    }
                }
                Some("ping") | Some("pong") | Some("connection_ack") => None,
                Some("complete") | Some("error") => {
                    if frame.get("type") == Some(&json!("error")) {
                        return Some(Err(ClientError::Graphql(
                            frame.get("payload").cloned().unwrap_or_default(),
                        )));
                    }
                    None
                }
                _ => None,
            }
        })
    }
}

/// Connects to `endpoint`'s WebSocket URL (http/https rewritten to
/// ws/wss via [`Endpoint::ws_url`]) and starts `subscription`. The
/// returned stream yields decoded `TangibleEvent`s and ends on
/// `complete`, socket close or the first protocol violation.
pub(crate) async fn subscribe(
    endpoint: &Endpoint,
    field: &str,
    query: &str,
    variables: Value,
) -> Result<impl Stream<Item = Result<TangibleEvent, ClientError>> + 'static, ClientError> {
    WebSocketClient::connect(endpoint)
        .await?
        .subscribe(field, query, variables)
        .await
}
