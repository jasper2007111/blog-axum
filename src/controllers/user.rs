use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use axum::{Extension, Json};
use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::errors::CustomError;
use crate::models::user::User;
use crate::models::response::ResponseData;

use serde::{Deserialize, Serialize};

use axum::extract::State;

use crate::AppState;

use axum::{http::{Request, HeaderMap}, extract::Form};
use axum::http::request::Parts;

// pub async fn all_users(Extension(pool): Extension<MySqlPool>) -> impl IntoResponse {
// pub async fn all_users(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {

pub async fn all_users(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let sql = "SELECT * FROM t_user ".to_string();

    let user_id = headers.get("x-user-id");
    if let Some(id) = user_id {
        tracing::info!("user_id: {:?}", id.clone().to_str());
    }
    
    let users = sqlx::query_as::<_, User>(&sql)
        .fetch_all(&state.db)
        .await
        .unwrap();

    (StatusCode::OK, Json(users))
}

pub async fn get_me(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let user_id = headers.get("x-user-id");
    if let Some(id) = user_id {
        let sql =format!("SELECT * FROM t_user Where id={:?}", id);
        let user = sqlx::query_as::<_, User>(&sql)
        .fetch_one(&state.db)
        .await
        .unwrap();

        return (StatusCode::OK, Json(ResponseData::of_success(user)));
    }
    
    (StatusCode::OK, Json(ResponseData::of_failure()))
}
