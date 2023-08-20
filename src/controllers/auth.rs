use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use axum::{Extension, Json};
use serde_json::{json, Value};
use sqlx::MySqlPool;

use crate::errors::CustomError;
use crate::models::user::{User, LoginModel, RegisterUserModel};

use serde::{Deserialize, Serialize};

use rand_core::OsRng;

use jsonwebtoken::encode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::EncodingKey;
use jsonwebtoken::Header;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

use axum::extract::State;

use crate::AppState;

pub const JWT_SECRET: &str = "https://AXUM.RS";

pub struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey<'static>,
}

impl Keys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret).into_static(),
        }
    }
    pub fn global() -> Self {
        Self::new(JWT_SECRET.as_bytes())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}


#[derive(Deserialize)]
pub struct AuthPayload {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Serialize)]
pub struct AuthBody {
    pub access_token: String,
    pub token_type: String,
}

impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: String::from("Bearer"),
        }
    }
}

pub async fn register(
    Extension(pool): Extension<MySqlPool>,
    Json(model): Json<RegisterUserModel>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("listening on {:?}", model);

    let user_exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM t_user WHERE username = ?)")
            .bind(model.username.to_owned().to_ascii_lowercase())
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format!("Database error: {}", e),
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

    if let Some(exists) = user_exists {
        if exists {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "User with that email already exists",
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(model.password.as_bytes(), &salt)
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Error while hashing password: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })
        .map(|hash| hash.to_string())?;

    let sql = format!("INSERT INTO t_user (username, password) VALUES ('{}', '{}')", model.username.to_string(), hashed_password);
    let _user = sqlx::query(
        &sql
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Database error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let user_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "user": "user"
    })});

    Ok(Json(user_response))
}

pub async fn login(
    State(state): State<AppState>,
    Json(login_model): Json<LoginModel>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let sql = format!("SELECT * FROM t_user WHERE username ='{}'", &login_model.username);
    let user = sqlx::query_as::<_, User>(
       &sql
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?
    .ok_or_else(|| {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Invalid email or password",
        });
        (StatusCode::BAD_REQUEST, Json(error_response))
    })?;

    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(login_model.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !is_valid {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Invalid email or password"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::minutes(60)).timestamp() as usize;
    let claims: TokenClaims = TokenClaims {
        sub: user.id.to_string(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )
    .unwrap();


    tracing::info!("listening on {:?}", login_model);

    // let token = encode(&Header::default(), &claims, &Keys::global().encoding)
    //     .map_err(|err| Json(err.to_string()))
    //     .unwrap();

    Ok((StatusCode::OK, Json(AuthBody::new(token))))
}
