use std::collections::HashMap;

use thiserror::Error;

pub mod http;
pub mod websocket;

use http::{HttpMethod, HttpPackage, HttpRequest, HttpResponse};
use websocket::{WebSocketMessage, WebSocketPackage, WebSocketSession};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageResponse {
    Http(HttpResponse),
    WebSocketSession(WebSocketSession),
    WebSocketMessage(WebSocketMessage),
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageCommand {
    HttpRequest(HttpRequest),
    WebSocketConnect { url: String },
    WebSocketSend { session_id: String, message: String },
    WebSocketReceive { session_id: String },
    WebSocketClose { session_id: String },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PackageError {
    #[error("unknown package '{name}'")]
    UnknownPackage { name: String },
    #[error("unsupported command for package '{package}'")]
    UnsupportedCommand { package: String },
    #[error("package execution failed: {message}")]
    ExecutionFailed { message: String },
}

pub trait Package {
    fn execute(&mut self, command: PackageCommand) -> Result<PackageResponse, PackageError>;
}

#[derive(Default)]
pub struct PackageManager {
    packages: HashMap<String, Box<dyn Package>>,
}

impl PackageManager {
    pub fn with_defaults() -> Self {
        let mut manager = Self::default();
        manager.register("http", Box::<HttpPackage>::default());
        manager.register("websocket", Box::<WebSocketPackage>::default());
        manager
    }

    pub fn register(&mut self, name: impl Into<String>, package: Box<dyn Package>) {
        self.packages.insert(name.into(), package);
    }

    pub fn execute(
        &mut self,
        package_name: &str,
        command: PackageCommand,
    ) -> Result<PackageResponse, PackageError> {
        let package =
            self.packages
                .get_mut(package_name)
                .ok_or_else(|| PackageError::UnknownPackage {
                    name: package_name.to_owned(),
                })?;

        package.execute(command)
    }
}

impl From<reqwest::Method> for HttpMethod {
    fn from(value: reqwest::Method) -> Self {
        match value {
            reqwest::Method::GET => HttpMethod::Get,
            reqwest::Method::POST => HttpMethod::Post,
            reqwest::Method::PUT => HttpMethod::Put,
            reqwest::Method::PATCH => HttpMethod::Patch,
            reqwest::Method::DELETE => HttpMethod::Delete,
            reqwest::Method::HEAD => HttpMethod::Head,
            reqwest::Method::OPTIONS => HttpMethod::Options,
            _ => HttpMethod::Get,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoPackage;

    impl Package for EchoPackage {
        fn execute(&mut self, command: PackageCommand) -> Result<PackageResponse, PackageError> {
            match command {
                PackageCommand::HttpRequest(request) => Ok(PackageResponse::Http(HttpResponse {
                    status: 200,
                    body: request.url,
                    headers: Default::default(),
                })),
                _ => Err(PackageError::UnsupportedCommand {
                    package: "echo".to_owned(),
                }),
            }
        }
    }

    #[test]
    fn returns_error_for_unknown_package() {
        let mut manager = PackageManager::default();

        let result = manager.execute(
            "missing",
            PackageCommand::HttpRequest(HttpRequest::new(HttpMethod::Get, "https://example.com")),
        );

        assert!(matches!(result, Err(PackageError::UnknownPackage { .. })));
    }

    #[test]
    fn delegates_to_registered_package() {
        let mut manager = PackageManager::default();
        manager.register("echo", Box::new(EchoPackage));

        let result = manager
            .execute(
                "echo",
                PackageCommand::HttpRequest(HttpRequest::new(
                    HttpMethod::Get,
                    "https://example.com",
                )),
            )
            .expect("echo package should handle command");

        assert_eq!(
            result,
            PackageResponse::Http(HttpResponse {
                status: 200,
                body: "https://example.com".to_owned(),
                headers: Default::default(),
            })
        );
    }
}
