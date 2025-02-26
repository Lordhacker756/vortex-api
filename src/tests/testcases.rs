use axum::http::Method;
use chrono::{Duration, Utc};
use serde_json::json;

use super::test_utils::setup_test_app;

#[tokio::test]
async fn test_poll_crud_operations() {
    let (app, _db) = setup_test_app().await;

    // Test Create Poll
    let create_poll_body = json!({
        "name": "Test Poll",
        "createdBy": "test_user_id",
        "isMulti": false,
        "startDate": Utc::now().to_rfc3339(),
        "endDate": (Utc::now() + Duration::days(1)).to_rfc3339(),
        "options": ["Option 1", "Option 2", "Option 3"]
    });

    let (status, response) =
        mock_authenticated_request(&app, "/api/polls", Method::POST, Some(create_poll_body)).await;

    assert_eq!(status.as_u16(), 201); // Changed to 201 for CREATED status
    let poll_id = response["data"]["pollId"]
        .as_str()
        .expect("Failed to get poll ID");

    // Test Get Poll by ID
    let (status, response) = mock_authenticated_request(
        &app,
        &format!("/api/polls/{}", poll_id),
        Method::GET,
        None::<()>,
    )
    .await;

    assert_eq!(status.as_u16(), 200);
    assert_eq!(response["data"]["name"].as_str().unwrap(), "Test Poll");

    // Test Update Poll
    let update_poll_body = json!({
        "name": "Updated Test Poll",
        "isMulti": true,
        "startDate": (Utc::now()).to_rfc3339(),
        "endDate": (Utc::now() + Duration::days(2)).to_rfc3339()
    });

    let (status, _) = mock_authenticated_request(
        &app,
        &format!("/api/polls/{}", poll_id),
        Method::PATCH,
        Some(update_poll_body),
    )
    .await;

    assert_eq!(status.as_u16(), 200);

    // Test Cast Vote
    let (status, _) = mock_authenticated_request(
        &app,
        &format!("/api/polls/{}/vote?option_id=option_1", poll_id),
        Method::GET,
        None::<()>,
    )
    .await;

    assert_eq!(status.as_u16(), 200);

    // Test Get Poll Results
    let (status, response) = mock_authenticated_request(
        &app,
        &format!("/api/polls/{}/results", poll_id),
        Method::GET,
        None::<()>,
    )
    .await;

    assert_eq!(status.as_u16(), 200);
    let results = response;
    assert!(results["data"]["options"]
        .as_array()
        .unwrap()
        .iter()
        .any(|opt| opt["votes"].as_i64().unwrap() > 0));

    // Test Close Poll
    let (status, _) = mock_authenticated_request(
        &app,
        &format!("/api/polls/{}/close", poll_id),
        Method::GET,
        None::<()>,
    )
    .await;

    assert_eq!(status.as_u16(), 200);
}

#[tokio::test]
async fn test_poll_validation() {
    let (app, _db) = setup_test_app().await;

    // Test Invalid Poll Creation (Past End Date)
    let invalid_poll_body = json!({
        "name": "Invalid Poll",
        "createdBy": "test_user_id",
        "isMulti": false,
        "startDate": (Utc::now() + Duration::days(1)).to_rfc3339(),
        "endDate": (Utc::now()).to_rfc3339(),
        "options": ["Option 1"]
    });

    let (status, _) =
        mock_authenticated_request(&app, "/api/polls", Method::POST, Some(invalid_poll_body)).await;

    assert_eq!(status.as_u16(), 400);

    // Test Create Poll with No Options
    let no_options_poll_body = json!({
        "name": "No Options Poll",
        "createdBy": "test_user_id",
        "isMulti": false,
        "startDate": (Utc::now()).to_rfc3339(),
        "endDate": (Utc::now() + Duration::days(1)).to_rfc3339(),
        "options": []
    });

    let (status, _) =
        mock_authenticated_request(&app, "/api/polls", Method::POST, Some(no_options_poll_body))
            .await;

    assert_eq!(status.as_u16(), 400);
}
