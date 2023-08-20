
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use axum::{Extension, Json};
use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::errors::CustomError;
use crate::models::article::{Article, CreateArticlePayload, EditArticlePayload};

use serde::{Deserialize, Serialize};

use axum::extract::State;
use axum::extract::Query;

use crate::AppState;

use axum::{http::{Request, HeaderMap}, extract::Form};
use axum::http::request::Parts;

#[derive(Deserialize)]
pub struct Pagination {
    pub cur_page: usize,
    pub page_size: usize,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
struct Row {
    count: i64
}

pub async fn list(State(state): State<AppState>, headers: HeaderMap, pagination: Query<Pagination>) -> impl IntoResponse {
    let sql = format!("SELECT * FROM t_article order by id DESC limit {}, {}", (pagination.cur_page-1)*pagination.page_size, pagination.page_size);
    tracing::info!("sql: {:?}", sql);

    let user_id = headers.get("x-user-id");
    if let Some(id) = user_id {
        tracing::info!("user_id: {:?}", id);
    }
    
    let users = sqlx::query_as::<_, Article>(&sql)
        .fetch_all(&state.db)
        .await
        .unwrap();

    let row = sqlx::query_as::<_, Row>("SELECT count(id) as count FROM t_article").fetch_one(&state.db).await.unwrap();
    tracing::info!("count: {:?}", row.count);

    (StatusCode::OK, Json(users))
}

pub async fn create(State(state): State<AppState>, headers: HeaderMap, Json(payload): Json<CreateArticlePayload>) -> impl IntoResponse {
    tracing::info!("payload: {:?}", payload);
    

    let user_id: &axum::http::HeaderValue = headers.get("x-user-id").unwrap();
    let sql = format!("insert into t_article(user_id, title, content, create_time) values ({}, '{}', '{}', now())", user_id.to_str().unwrap(), payload.title, payload.content);
    tracing::info!("sqlv: {:?}", sql);

    let rep = sqlx::query(&sql).execute(&state.db).await.unwrap();
    
    // let users = sqlx::query_as::<_, Article>(&sql)
    //     .fetch_all(&state.db)
    //     .await
    //     .unwrap();

    (StatusCode::OK, Json(""))
}

pub async fn get_article(Path(id): Path<String>, State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    tracing::info!("article_id: {:?}", id);
    

    let user_id: &axum::http::HeaderValue = headers.get("x-user-id").unwrap();
    let sql = format!("Select * From t_article Where id={}", id);
    tracing::info!("sql: {:?}", sql);

    let rep = sqlx::query_as::<_, Article>(&sql).fetch_one(&state.db).await.unwrap();
    
    // let users = sqlx::query_as::<_, Article>(&sql)
    //     .fetch_all(&state.db)
    //     .await
    //     .unwrap();

    (StatusCode::OK, Json(rep))
}

pub async fn edit(State(state): State<AppState>, headers: HeaderMap, Json(payload): Json<EditArticlePayload>) -> impl IntoResponse {
    tracing::info!("payload: {:?}", payload);
    
    let user_id: &axum::http::HeaderValue = headers.get("x-user-id").unwrap();
    let sql = format!("UPDATE t_article SET title='{}', content='{}', update_time=now() Where id={}", payload.title ,payload.content, payload.id);
    tracing::info!("sqlv: {:?}", sql);

    let rep = sqlx::query(&sql).execute(&state.db).await.unwrap();
    
    // let users = sqlx::query_as::<_, Article>(&sql)
    //     .fetch_all(&state.db)
    //     .await
    //     .unwrap();

    (StatusCode::OK, Json(""))
}
