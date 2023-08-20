use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,

    #[serde(skip_serializing)]
    pub password: String
}

#[derive(Deserialize, Serialize, sqlx::FromRow, Debug, Clone)]
pub struct LoginModel {
    pub username: String,
    pub password: String,
}


#[derive(Deserialize, Serialize, sqlx::FromRow, Debug, Clone)]
pub struct RegisterUserModel {
    pub username: String,
    pub password: String,
}
