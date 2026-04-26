use std::collections::HashMap;

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::packages::{Package, PackageCommand, PackageError, PackageResponse};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
}

impl HttpRequest {
    pub fn new(method: HttpMethod, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            body: None,
            headers: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

pub trait HttpTransport {
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse, PackageError>;
}

#[derive(Default)]
pub struct ReqwestTransport {
    client: Client,
}

impl HttpTransport for ReqwestTransport {
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse, PackageError> {
        let mut request_builder = self
            .client
            .request(to_reqwest_method(request.method), &request.url);

        let headers = to_reqwest_headers(&request.headers)?;
        request_builder = request_builder.headers(headers);

        if let Some(body) = &request.body {
            request_builder = request_builder.body(body.clone());
        }

        let response = request_builder
            .send()
            .map_err(|error| PackageError::ExecutionFailed {
                message: error.to_string(),
            })?;

        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.to_string(),
                    value.to_str().unwrap_or_default().to_owned(),
                )
            })
            .collect();
        let body = response
            .text()
            .map_err(|error| PackageError::ExecutionFailed {
                message: error.to_string(),
            })?;

        Ok(HttpResponse {
            status,
            body,
            headers,
        })
    }
}

pub struct HttpPackage {
    transport: Box<dyn HttpTransport + Send + Sync>,
}

impl Default for HttpPackage {
    fn default() -> Self {
        Self::new(Box::<ReqwestTransport>::default())
    }
}

impl HttpPackage {
    pub fn new(transport: Box<dyn HttpTransport + Send + Sync>) -> Self {
        Self { transport }
    }
}

impl Package for HttpPackage {
    fn execute(&mut self, command: PackageCommand) -> Result<PackageResponse, PackageError> {
        let PackageCommand::HttpRequest(request) = command else {
            return Err(PackageError::UnsupportedCommand {
                package: "http".to_owned(),
            });
        };

        let response = self.transport.send(&request)?;
        Ok(PackageResponse::Http(response))
    }
}

fn to_reqwest_method(method: HttpMethod) -> reqwest::Method {
    match method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Head => reqwest::Method::HEAD,
        HttpMethod::Options => reqwest::Method::OPTIONS,
    }
}

fn to_reqwest_headers(headers: &HashMap<String, String>) -> Result<HeaderMap, PackageError> {
    let mut mapped = HeaderMap::new();
    for (name, value) in headers {
        let header_name = HeaderName::from_bytes(name.as_bytes()).map_err(|error| {
            PackageError::ExecutionFailed {
                message: error.to_string(),
            }
        })?;
        let header_value =
            HeaderValue::from_str(value).map_err(|error| PackageError::ExecutionFailed {
                message: error.to_string(),
            })?;
        mapped.insert(header_name, header_value);
    }

    Ok(mapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeTransport;

    impl HttpTransport for FakeTransport {
        fn send(&self, request: &HttpRequest) -> Result<HttpResponse, PackageError> {
            Ok(HttpResponse {
                status: 201,
                body: format!(
                    "{} {}",
                    request.url,
                    request.body.clone().unwrap_or_default()
                ),
                headers: HashMap::from([("x-source".to_owned(), "fake".to_owned())]),
            })
        }
    }

    #[test]
    fn executes_http_request_via_transport() {
        let mut package = HttpPackage::new(Box::new(FakeTransport));
        let mut request = HttpRequest::new(HttpMethod::Post, "https://example.com");
        request.body = Some("payload".to_owned());

        let response = package
            .execute(PackageCommand::HttpRequest(request))
            .expect("http request should succeed");

        assert_eq!(
            response,
            PackageResponse::Http(HttpResponse {
                status: 201,
                body: "https://example.com payload".to_owned(),
                headers: HashMap::from([("x-source".to_owned(), "fake".to_owned())]),
            })
        );
    }

    #[test]
    fn rejects_non_http_commands() {
        let mut package = HttpPackage::new(Box::new(FakeTransport));

        let result = package.execute(PackageCommand::WebSocketConnect {
            url: "ws://localhost".to_owned(),
        });

        assert!(matches!(
            result,
            Err(PackageError::UnsupportedCommand { package }) if package == "http"
        ));
    }
}
