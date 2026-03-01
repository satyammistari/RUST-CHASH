use std::{fmt, str::FromStr};

use axum::body::Body;
use axum::http::{header::HOST, uri::{PathAndQuery, Scheme, Uri}};
use hyper::{Request, Response};
use hyper_util::{client::legacy::{connect::HttpConnector, Client}, rt::TokioExecutor};
use http_body_util::BodyExt;
use tracing::debug;

#[derive(Clone)]
pub struct ProxyClient {
	inner: Client<HttpConnector, Body>,
}

impl ProxyClient {
	pub fn new() -> Self {
		let mut connector = HttpConnector::new();
		connector.enforce_http(false);
		let client = Client::builder(TokioExecutor::new()).build(connector);
		Self { inner: client }
	}

	pub async fn forward(
		&self,
		mut req: Request<Body>,
		target: &str,
	) -> Result<Response<Body>, ProxyError> {
		let target_uri = Uri::from_str(target).map_err(|_| ProxyError::InvalidBackend(target.to_string()))?;
		let scheme = target_uri.scheme().cloned().unwrap_or_else(|| Scheme::HTTP);
		let authority = target_uri
			.authority()
			.cloned()
			.ok_or_else(|| ProxyError::InvalidBackend(target.to_string()))?;

		let path_and_query = req
			.uri()
			.path_and_query()
			.cloned()
			.unwrap_or_else(|| PathAndQuery::from_static("/"));

		let new_uri = Uri::builder()
			.scheme(scheme.clone())
			.authority(authority.as_str())
			.path_and_query(path_and_query)
			.build()
			.map_err(|_| ProxyError::InvalidBackend(target.to_string()))?;

		*req.uri_mut() = new_uri;
		req.headers_mut()
			.insert(HOST, authority.as_str().parse().map_err(|_| ProxyError::InvalidBackend(target.to_string()))?);

		debug!("forwarding request to {}", req.uri());

		let response = self
			.inner
			.request(req)
			.await
			.map_err(ProxyError::Http)?;
		let (parts, body) = response.into_parts();
		let axum_body = Body::from_stream(body.into_data_stream());
		Ok(Response::from_parts(parts, axum_body))
	}
}

#[derive(Debug)]
pub enum ProxyError {
	InvalidBackend(String),
	Http(hyper_util::client::legacy::Error),
}

impl fmt::Display for ProxyError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ProxyError::InvalidBackend(uri) => write!(f, "invalid backend URI: {}", uri),
			ProxyError::Http(err) => write!(f, "http error: {}", err),
		}
	}
}

impl std::error::Error for ProxyError {}
