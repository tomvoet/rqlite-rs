//! Contains integration tests with other libraries and frameworks
use std::sync::Arc;

use rqlite_rs::RqliteClient;

use axum::{extract::State, http::StatusCode, routing::get, Router};
use axum_test::TestServer;

mod common;

#[tokio::test]
async fn integration_with_axum() {
    #[derive(Clone)]
    struct AppState {
        rqlite_client: Arc<RqliteClient>,
    }

    async fn example_endpoint(State(AppState { rqlite_client }): State<AppState>) -> StatusCode {
        match rqlite_client.nodes().await {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    let rqlite_client = common::get_client();
    let state = AppState {
        rqlite_client: Arc::new(rqlite_client),
    };

    let app = Router::new()
        .route("/", get(example_endpoint))
        .with_state(state);

    let server = TestServer::new(app).unwrap();

    let response = server.get("/").await;

    assert_eq!(response.status_code(), StatusCode::OK);
}
