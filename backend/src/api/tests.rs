#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    };
    use tower::ServiceExt;
    use crate::api::{request_id_middleware, REQUEST_ID_HEADER};

    fn test_app() -> Router {
        Router::new()
            .route("/test", get(|| async { "OK" }))
            .layer(axum::middleware::from_fn(request_id_middleware))
    }

    #[tokio::test]
    async fn test_provided_request_id_accepted() {
        let app = test_app();
        let request_id = "test-request-id-12345";

        let request = Request::builder()
            .uri("/test")
            .header(REQUEST_ID_HEADER, request_id)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let returned_id = response.headers().get(REQUEST_ID_HEADER).unwrap().to_str().unwrap();
        assert_eq!(returned_id, request_id);
    }

    #[tokio::test]
    async fn test_missing_request_id_generates_uuid() {
        let app = test_app();

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let returned_id = response.headers().get(REQUEST_ID_HEADER).unwrap().to_str().unwrap();
        assert!(!returned_id.is_empty());
        assert_eq!(returned_id.len(), 36); // UUID length
    }

    #[tokio::test]
    async fn test_invalid_too_long_request_id_generates_uuid() {
        let app = test_app();
        let long_id = "a".repeat(130);

        let request = Request::builder()
            .uri("/test")
            .header(REQUEST_ID_HEADER, long_id)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let returned_id = response.headers().get(REQUEST_ID_HEADER).unwrap().to_str().unwrap();
        assert!(!returned_id.is_empty());
        assert_eq!(returned_id.len(), 36); // Fallback to UUID
    }

    #[tokio::test]
    async fn test_invalid_empty_request_id_generates_uuid() {
        let app = test_app();

        let request = Request::builder()
            .uri("/test")
            .header(REQUEST_ID_HEADER, "")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let returned_id = response.headers().get(REQUEST_ID_HEADER).unwrap().to_str().unwrap();
        assert!(!returned_id.is_empty());
        assert_eq!(returned_id.len(), 36); // Fallback to UUID
    }
}
