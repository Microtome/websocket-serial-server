use crate::messages::SerialResponse;
use anyhow::*;
use hyper_tungstenite::tungstenite::error::ProtocolError;
use hyper_tungstenite::tungstenite;
use thiserror::Error;

/// The kinds of errors that WebsocketSerialServer can return
/// TODO Rename to WsspsError
#[derive(Error, Debug)]
pub enum WebsocketSerialServerError {
    /// Unknown server request
    #[error("Unknown request")]
    UnknownRequest,
    /// Open port not found
    #[error("Open port '{port}' not found")]
    OpenPortNotFound { port: String },
    #[error("Subscription '{subscription_id}' not found")]
    /// Subscription not found
    SubscriptionNotFound { subscription_id: String },
    /// Port already write locked error
    #[error("Port '{port}' is already writelocked")]
    AlreadyWriteLocked { port: String },
    /// Need write lock error
    #[error("Port '{port}' needs to be writelocked before writing")]
    NeedWriteLock { port: String },
    /// Serial port read error
    #[error("Port '{port}' had read error")]
    PortReadError { port: String },
    /// Serial Port EOF error
    #[error("Port '{port}' EOF")]
    PortEofError { port: String },
    /// Serial port write error
    #[error("Port '{port}' write error")]
    PortWriteError { port: String },
    /// Send to subscriber error
    #[error("Failed send to subscriber '{subscription_id}'")]
    SubscriberSendError { subscription_id: String },
    /// Catchall for all others
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<ProtocolError> for WebsocketSerialServerError {
    fn from(protocol_error: ProtocolError) -> Self {
        Self::Other(anyhow!(protocol_error))
    }
}

impl From<tungstenite::Error> for WebsocketSerialServerError {
    fn from(tungstenite_error: tungstenite::Error) -> Self {
        Self::Other(anyhow!(tungstenite_error))
    }
}

pub type Result<T> = std::result::Result<T, WebsocketSerialServerError>;

impl From<WebsocketSerialServerError> for SerialResponse {
    fn from(wss_error: WebsocketSerialServerError) -> Self {
        SerialResponse::Error {
            error: wss_error.to_string(),
        }
    }
}
