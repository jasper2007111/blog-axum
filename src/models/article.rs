use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct Article {
    pub id: i64,
    pub user_id: i64,

    pub title: String,
    pub content:String,

    pub create_time: chrono::NaiveDateTime,
    pub update_time: Option<chrono::NaiveDateTime>,
}



#[derive(Deserialize, Serialize, sqlx::FromRow, Debug)]
pub struct CreateArticlePayload {
    pub title: String,
    pub content:String
}

#[derive(Deserialize, Serialize, sqlx::FromRow, Debug)]
pub struct EditArticlePayload {
    pub id: i64,
    pub title: String,
    pub content:String
}


