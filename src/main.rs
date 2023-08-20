use axum::{
    extract::{Extension, State},
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use jsonwebtoken::decode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;

use tower_http::cors::{Any, CorsLayer};
// use tower_http::header::{CONTENT_TYPE};

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};

use sqlx::MySql;
use sqlx::MySqlPool;

mod controllers;
mod errors;
mod models;

use sqlx::Pool;

#[derive(Clone)]
pub struct AppState {
    db: Pool<MySql>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

async fn auth<B>(
    State(state): State<AppState>,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_owned())
            } else {
                None
            }
        });

    let token = token.ok_or_else(|| {
        let json_error = ErrorResponse {
            status: "fail",
            message: "You are not logged in, please provide token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;

    let claims = decode::<controllers::auth::TokenClaims>(
        &token,
        &DecodingKey::from_secret(controllers::auth::JWT_SECRET.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| {
        let json_error = ErrorResponse {
            status: "fail",
            message: "Invalid token".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?
    .claims;

    tracing::info!("token: {}", claims.sub);

    let sql = format!("SELECT * FROM t_user WHERE id = {}", claims.sub);
    let user = sqlx::query_as::<_, models::user::User>(&sql)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            let json_error = ErrorResponse {
                status: "fail",
                message: format!("Error fetching user from database: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json_error))
        })?;

    let user = user.ok_or_else(|| {
        let json_error = ErrorResponse {
            status: "fail",
            message: "The user belonging to this token no longer exists".to_string(),
        };
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;

    // request.extensions_mut().insert(user);
    request
        .headers_mut()
        .insert("x-user-id", HeaderValue::from(user.id));
    Ok(next.run(request).await)
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "没有找到URL")
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // initialize tracing
    // tracing_subscriber::registry()
    //     .with(tracing_subscriber::EnvFilter::new(
    //         std::env::var("tower_http=trace")
    //             .unwrap_or_else(|_| "blog_axum_logging=debug,tower_http=debug".into()),
    //     ))
    //     .with(tracing_subscriber::fmt::layer())
    //     .init();

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let pool = MySqlPool::connect("mysql://root:123456@127.0.0.1/blog").await?;

    // let state = Arc::new(
    //     AppState{
    //         db: pool
    //     }
    // );

    let state = AppState { db: pool.clone() };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers([CONTENT_TYPE, AUTHORIZATION, ACCEPT]);

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/api/auth/register", post(controllers::auth::register))
        .route("/api/auth/login", post(controllers::auth::login))
        .route(
            "/api/user/list",
            get(controllers::user::all_users)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth)),
        )
        .route(
            "/api/user/me",
            get(controllers::user::get_me)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth)),
        )
        .route(
            "/api/article/list",
            get(controllers::article::list)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth)),
        )
        .route(
            "/api/article/create",
            post(controllers::article::create)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth)),
        )
        .route(
            "/api/article/:id",
            get(controllers::article::get_article)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth)),
        )
        .route(
            "/api/article/edit",
            post(controllers::article::edit)
                .route_layer(middleware::from_fn_with_state(state.clone(), auth)),
        )
        // .layer(
        //     TraceLayer::new_for_http()
        //         .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
        //         .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        //         )
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .fallback(handler_404);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}