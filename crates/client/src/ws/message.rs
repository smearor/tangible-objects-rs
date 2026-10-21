//! Frame conversion between `tokio-tungstenite`'s `Message` and
//! `graphql-ws-client`'s [`Message`] - `From` impls are impossible
//! (both types are foreign), so the mapping lives in free
//! functions.

use tokio_tungstenite::tungstenite;

/// Maps an inbound tungstenite frame to the protocol [`Message`];
/// `Binary`/`Frame` frames carry no subscription payload and drop.
pub(crate) fn to_protocol(frame: tungstenite::Message) -> Option<graphql_ws_client::Message> {
    match frame {
        tungstenite::Message::Text(text) => {
            Some(graphql_ws_client::Message::Text(text.to_string()))
        }
        tungstenite::Message::Ping(_) => Some(graphql_ws_client::Message::Ping),
        tungstenite::Message::Pong(_) => Some(graphql_ws_client::Message::Pong),
        tungstenite::Message::Close(frame) => {
            let (code, reason) = frame
                .map(|f| (Some(u16::from(f.code)), Some(f.reason.to_string())))
                .unwrap_or_default();
            Some(graphql_ws_client::Message::Close { code, reason })
        }
        _ => None,
    }
}

/// Maps an outbound protocol [`Message`] to a tungstenite frame.
pub(crate) fn into_frame(message: graphql_ws_client::Message) -> tungstenite::Message {
    match message {
        graphql_ws_client::Message::Text(text) => tungstenite::Message::Text(text.into()),
        graphql_ws_client::Message::Ping => tungstenite::Message::Ping(Default::default()),
        graphql_ws_client::Message::Pong => tungstenite::Message::Pong(Default::default()),
        graphql_ws_client::Message::Close { code, reason } => {
            tungstenite::Message::Close(code.map(|code| tungstenite::protocol::CloseFrame {
                code: code.into(),
                reason: reason.unwrap_or_default().into(),
            }))
        }
    }
}
