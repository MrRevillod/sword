use axum::http::Extensions as HttpExtensions;
use bytes::Bytes;
use serde::{Serialize, de::DeserializeOwned};
use socketioxide::{
    ParserError, ProtocolVersion, SendError, SocketError, TransportType,
    adapter::{Adapter, LocalAdapter},
    extensions::Extensions,
    extract::{AckSender, Event, SocketRef},
    handler::{FromConnectParts, FromDisconnectParts, FromMessageParts},
    socket::{DisconnectReason, Socket},
};

use socketioxide_core::{Sid, Value, parser::ParseError};
use std::{convert::Infallible, sync::Arc};

use crate::prelude::SocketIoParser;

/// A unified extractor that combines multiple socketioxide extractors into a single context.
///
/// Provides access to socket operations, message data, acknowledgments, event names,
/// and disconnect reasons depending on the handler type.
pub struct SocketContext<A: Adapter = LocalAdapter> {
    socket: SocketRef<A>,
    data: Option<Value>,
    ack: Option<AckSender<A>>,
    disconnect_reason: Option<DisconnectReason>,
    event: Option<Box<str>>,
}

impl<A> SocketContext<A>
where
    A: Adapter,
{
    /// The `SocketRef` extractor equivalent method.
    pub fn socket(&self) -> &Socket<A> {
        &self.socket
    }

    /// The `TryData<T>` extractor equivalent method.
    ///   
    /// Deserializes message data to the specified type.
    pub fn try_data<T: DeserializeOwned>(&self) -> Result<T, ParserError> {
        let Some(data) = &self.data else {
            return Err(ParserError::new(ParseError::InvalidData));
        };

        let parser = self
            .socket
            .req_parts()
            .extensions
            .get::<SocketIoParser>()
            .unwrap_or(&SocketIoParser::Common);

        let bytes = match &data {
            Value::Str(s, _) => s.as_ref(),
            Value::Bytes(b) => b.as_ref(),
        };

        match parser {
            SocketIoParser::Common => serde_json::from_slice(bytes)
                .map_err(|_| ParserError::new(ParseError::InvalidData)),
            SocketIoParser::MsgPack => rmp_serde::from_slice(bytes)
                .map_err(|_| ParserError::new(ParseError::InvalidData)),
        }
    }

    /// The `Event` extractor equivalent method.
    ///
    /// Returns the event name for message handlers.
    /// Returns `None` for connect/disconnect handlers (protocol-level events).
    pub fn event(&self) -> Option<&str> {
        self.event.as_deref()
    }

    /// The `AckSender` `send` method equivalent.
    ///
    /// Sends an acknowledgment response to the client.
    pub fn ack<D>(self, data: &D) -> Result<(), SendError>
    where
        D: Serialize + ?Sized,
    {
        let Some(ack) = self.ack else {
            return Err(SendError::Socket(SocketError::Closed));
        };

        ack.send(data)?;

        Ok(())
    }

    pub fn id(&self) -> &Sid {
        &self.socket.id
    }

    /// Checks if an acknowledgment sender is available.
    pub fn has_ack(&self) -> bool {
        self.ack.is_some()
    }

    /// Checks if data is still available (not consumed by `try_data()`).
    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }

    /// Returns access to the socket's extension storage.
    pub fn extensions(&self) -> &Extensions {
        &self.socket.extensions
    }

    /// Returns access to HTTP request extensions.
    pub fn http_extensions(&self) -> &HttpExtensions {
        &self.socket.req_parts().extensions
    }

    /// Get the Socket.IO protocol version used by the client.
    pub fn protocol_version(&self) -> ProtocolVersion {
        self.socket.protocol()
    }

    /// Returns the transport type used by the client (WebSocket or polling).
    pub fn transport_type(&self) -> TransportType {
        self.socket.transport_type()
    }

    /// Disconnects the socket and triggers the disconnect handler.
    pub fn disconnect(self) -> Result<(), SocketError> {
        self.socket.disconnect()
    }

    /// Returns the reason for socket disconnection if this context was created from a disconnect event.
    ///
    /// **Returns `None` for:**
    /// - **Connect handlers**: No disconnection has occurred yet
    /// - **Message handlers**: The socket is still connected and processing messages
    ///
    /// **Returns `Some(reason)` for:**
    /// - **Disconnect handlers**: Provides the specific reason why the socket disconnected
    ///   (e.g., client disconnect, server disconnect, transport error, etc.)
    pub fn disconnect_reason(&self) -> Option<&DisconnectReason> {
        self.disconnect_reason.as_ref()
    }
}

impl<A> FromMessageParts<A> for SocketContext<A>
where
    A: Adapter,
{
    type Error = Infallible;

    fn from_message_parts(
        s: &Arc<Socket<A>>,
        v: &mut Value,
        ack_id: &Option<i64>,
    ) -> Result<Self, Self::Error> {
        let ack = ack_id
            .and_then(|id| AckSender::from_message_parts(s, v, &Some(id)).ok());

        let event = Event::from_message_parts(s, v, ack_id)
            .ok()
            .map(|e| e.0.into_boxed_str());

        let data = std::mem::replace(v, Value::Bytes(Bytes::new()));
        let socket_ref = SocketRef::from_message_parts(s, v, ack_id)?;

        Ok(SocketContext {
            socket: socket_ref,
            data: Some(data),
            ack,
            disconnect_reason: None,
            event,
        })
    }
}

impl<A> FromConnectParts<A> for SocketContext<A>
where
    A: Adapter,
{
    type Error = Infallible;

    fn from_connect_parts(
        s: &Arc<Socket<A>>,
        auth: &Option<Value>,
    ) -> Result<Self, Self::Error> {
        Ok(SocketContext {
            socket: SocketRef::from_connect_parts(s, auth)?,
            data: auth.clone(),
            ack: None,
            disconnect_reason: None,
            event: None,
        })
    }
}

impl<A> FromDisconnectParts<A> for SocketContext<A>
where
    A: Adapter,
{
    type Error = Infallible;

    fn from_disconnect_parts(
        s: &Arc<Socket<A>>,
        reason: DisconnectReason,
    ) -> Result<Self, Self::Error> {
        Ok(SocketContext {
            socket: SocketRef::from_disconnect_parts(s, reason)?,
            data: None,
            ack: None,
            disconnect_reason: Some(reason),
            event: None,
        })
    }
}
