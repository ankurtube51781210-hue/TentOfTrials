use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use uuid::Uuid;
use tracing::Instrument;

const REQUEST_ID_HEADER: &str = "x-request-id";

/// Starts the HTTP API server.
pub async fn start_server(port: u16) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/health", get(health_check))
        .layer(axum::middleware::from_fn(request_id_middleware));

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("starting HTTP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn request_id_middleware(mut request: Request<Body>, next: Next) -> Response {
    let mut request_id = None;

    if let Some(header_val) = request.headers().get(REQUEST_ID_HEADER) {
        if let Ok(header_str) = header_val.to_str() {
            let header_str = header_str.trim();
            if !header_str.is_empty() && header_str.len() < 128 {
                request_id = Some(header_str.to_string());
            }
        }
    }

    let request_id = request_id.unwrap_or_else(|| Uuid::new_v4().to_string());

    // Insert into request extensions so other handlers can access it if needed
    request.extensions_mut().insert(request_id.clone());

    // Create a tracing span with the request_id
    let span = tracing::info_span!("http_request", request_id = %request_id);

    // Call the next handler within the span
    let mut response = async move { next.run(request).await }.instrument(span).await;

    // Add the request ID to the response headers
    if let Ok(header_val) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(REQUEST_ID_HEADER, header_val);
    }

    response
}

mod tests;
