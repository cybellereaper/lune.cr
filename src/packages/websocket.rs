use std::collections::HashMap;

use tungstenite::{connect, Message};

use crate::packages::{Package, PackageCommand, PackageError, PackageResponse};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketSession {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSocketMessage {
    pub payload: String,
}

pub trait WebSocketTransport {
    fn connect(&mut self, url: &str) -> Result<WebSocketSession, PackageError>;
    fn send_text(&mut self, session_id: &str, message: &str) -> Result<(), PackageError>;
    fn receive_text(&mut self, session_id: &str) -> Result<WebSocketMessage, PackageError>;
    fn close(&mut self, session_id: &str) -> Result<(), PackageError>;
}

#[derive(Default)]
pub struct TungsteniteTransport {
    sockets: HashMap<
        String,
        tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>,
    >,
}

impl WebSocketTransport for TungsteniteTransport {
    fn connect(&mut self, url: &str) -> Result<WebSocketSession, PackageError> {
        let (socket, _response) = connect(url).map_err(|error| PackageError::ExecutionFailed {
            message: error.to_string(),
        })?;
        let session = WebSocketSession { id: url.to_owned() };

        self.sockets.insert(session.id.clone(), socket);
        Ok(session)
    }

    fn send_text(&mut self, session_id: &str, message: &str) -> Result<(), PackageError> {
        let socket = self.socket_mut(session_id)?;
        socket
            .send(Message::Text(message.to_owned().into()))
            .map_err(|error| PackageError::ExecutionFailed {
                message: error.to_string(),
            })
    }

    fn receive_text(&mut self, session_id: &str) -> Result<WebSocketMessage, PackageError> {
        let socket = self.socket_mut(session_id)?;
        let message = socket
            .read()
            .map_err(|error| PackageError::ExecutionFailed {
                message: error.to_string(),
            })?;

        let payload = message
            .into_text()
            .map_err(|error| PackageError::ExecutionFailed {
                message: error.to_string(),
            })?
            .to_string();

        Ok(WebSocketMessage { payload })
    }

    fn close(&mut self, session_id: &str) -> Result<(), PackageError> {
        let mut socket =
            self.sockets
                .remove(session_id)
                .ok_or_else(|| PackageError::ExecutionFailed {
                    message: format!("unknown websocket session '{session_id}'"),
                })?;

        socket
            .close(None)
            .map_err(|error| PackageError::ExecutionFailed {
                message: error.to_string(),
            })
    }
}

impl TungsteniteTransport {
    fn socket_mut(
        &mut self,
        session_id: &str,
    ) -> Result<
        &mut tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>,
        PackageError,
    > {
        self.sockets
            .get_mut(session_id)
            .ok_or_else(|| PackageError::ExecutionFailed {
                message: format!("unknown websocket session '{session_id}'"),
            })
    }
}

pub struct WebSocketPackage {
    transport: Box<dyn WebSocketTransport + Send + Sync>,
}

impl Default for WebSocketPackage {
    fn default() -> Self {
        Self::new(Box::<TungsteniteTransport>::default())
    }
}

impl WebSocketPackage {
    pub fn new(transport: Box<dyn WebSocketTransport + Send + Sync>) -> Self {
        Self { transport }
    }
}

impl Package for WebSocketPackage {
    fn execute(&mut self, command: PackageCommand) -> Result<PackageResponse, PackageError> {
        match command {
            PackageCommand::WebSocketConnect { url } => {
                let session = self.transport.connect(&url)?;
                Ok(PackageResponse::WebSocketSession(session))
            }
            PackageCommand::WebSocketSend {
                session_id,
                message,
            } => {
                self.transport.send_text(&session_id, &message)?;
                Ok(PackageResponse::Empty)
            }
            PackageCommand::WebSocketReceive { session_id } => {
                let message = self.transport.receive_text(&session_id)?;
                Ok(PackageResponse::WebSocketMessage(message))
            }
            PackageCommand::WebSocketClose { session_id } => {
                self.transport.close(&session_id)?;
                Ok(PackageResponse::Empty)
            }
            _ => Err(PackageError::UnsupportedCommand {
                package: "websocket".to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct FakeWebSocketTransport {
        sessions: HashMap<String, Vec<String>>,
    }

    impl WebSocketTransport for FakeWebSocketTransport {
        fn connect(&mut self, url: &str) -> Result<WebSocketSession, PackageError> {
            self.sessions.insert(url.to_owned(), vec![]);
            Ok(WebSocketSession { id: url.to_owned() })
        }

        fn send_text(&mut self, session_id: &str, message: &str) -> Result<(), PackageError> {
            let session =
                self.sessions
                    .get_mut(session_id)
                    .ok_or_else(|| PackageError::ExecutionFailed {
                        message: "missing session".to_owned(),
                    })?;
            session.push(message.to_owned());
            Ok(())
        }

        fn receive_text(&mut self, session_id: &str) -> Result<WebSocketMessage, PackageError> {
            let session =
                self.sessions
                    .get_mut(session_id)
                    .ok_or_else(|| PackageError::ExecutionFailed {
                        message: "missing session".to_owned(),
                    })?;
            let payload = session.pop().unwrap_or_default();
            Ok(WebSocketMessage { payload })
        }

        fn close(&mut self, session_id: &str) -> Result<(), PackageError> {
            self.sessions.remove(session_id);
            Ok(())
        }
    }

    #[test]
    fn supports_connect_send_receive_and_close() {
        let mut package = WebSocketPackage::new(Box::<FakeWebSocketTransport>::default());

        let connected = package
            .execute(PackageCommand::WebSocketConnect {
                url: "ws://example".to_owned(),
            })
            .expect("connect should work");
        assert_eq!(
            connected,
            PackageResponse::WebSocketSession(WebSocketSession {
                id: "ws://example".to_owned()
            })
        );

        package
            .execute(PackageCommand::WebSocketSend {
                session_id: "ws://example".to_owned(),
                message: "hello".to_owned(),
            })
            .expect("send should work");

        let received = package
            .execute(PackageCommand::WebSocketReceive {
                session_id: "ws://example".to_owned(),
            })
            .expect("receive should work");

        assert_eq!(
            received,
            PackageResponse::WebSocketMessage(WebSocketMessage {
                payload: "hello".to_owned()
            })
        );

        package
            .execute(PackageCommand::WebSocketClose {
                session_id: "ws://example".to_owned(),
            })
            .expect("close should work");
    }
}
