use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HealthResponse {
    pub status: String,
}

/// Health check endpoint
///
/// Returns a simple health status to verify the server is running.
pub async fn health_check() -> (StatusCode, Json<HealthResponse>) {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "healthy".to_string(),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_returns_200_ok() {
        // Arrange: No setup needed for health check

        // Act: Call the health check handler
        let (status, Json(response)) = health_check().await;

        // Assert: Status should be 200 OK
        assert_eq!(status, StatusCode::OK);

        // Assert: Response should be valid JSON with correct structure
        assert_eq!(response.status, "healthy");
    }

    #[tokio::test]
    async fn test_health_check_response_structure() {
        // Arrange & Act
        let (_, Json(response)) = health_check().await;

        // Assert: Response should match expected structure
        let expected = HealthResponse {
            status: "healthy".to_string(),
        };
        assert_eq!(response, expected);
    }

    #[tokio::test]
    async fn test_health_check_is_fast() {
        // Arrange
        let start = std::time::Instant::now();

        // Act
        let _ = health_check().await;

        // Assert: Should execute in less than 10ms
        let duration = start.elapsed();
        assert!(duration.as_millis() < 10, "Health check took {:?}, expected < 10ms", duration);
    }
}
