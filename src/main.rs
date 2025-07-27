mod api;
mod colors;
mod config;
mod pages;
mod scheduler;
mod sqlite;
mod time;
mod utils;
mod youtube;

use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

use anyhow::Result;
use axum::{
    Router,
    extract::Query,
    http::{HeaderMap, StatusCode, header::CONTENT_TYPE},
    response::Redirect,
    routing::get,
};
use axum_extra::{TypedHeader, headers::UserAgent};
use config::CONFIG;
use dotenv::dotenv;
use pages::{PageContext, Pages, Render};
use reqwest::Method;
use tower_http::cors;
use utils::is_term;

#[derive(Clone)]
struct AppState {}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt().with_max_level(CONFIG.server.log_level).init();

    sqlite::init_db();
    scheduler::init_scheduler().await.expect("failed to init scheduler");

    let cors = cors::CorsLayer::new() //
        .allow_methods([Method::GET])
        .allow_origin(cors::Any);

    let state = AppState {};

    let router = Router::new() //
        .fallback(Redirect::temporary(&CONFIG.browser_redirect)) // This should handle term too
        .route("/", get(get_root))
        .route("/list", get(get_list))
        .layer(cors)
        .with_state(state);

    let host = Ipv4Addr::from_str(&CONFIG.server.host).expect("invalid host");
    let socket = SocketAddr::from((host, CONFIG.server.port));
    let listener = tokio::net::TcpListener::bind(&socket).await?;

    tracing::info!("listening on http://{}", &socket);

    axum::serve(
        listener, //
        router.into_make_service(),
    )
    .await?;

    Ok(())
}

// GET /
async fn get_root(
    query: Query<HashMap<String, String>>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
) -> impl axum::response::IntoResponse {
    let mut headers = HeaderMap::new();

    if !is_term(&user_agent) {
        headers.insert("Location", CONFIG.browser_redirect.parse().unwrap());
        return (StatusCode::TEMPORARY_REDIRECT, headers, "Redirecting to GitHub...".to_string());
    }

    let oshi = query.get("oshi").cloned();

    let id: &Option<String> = if oshi.is_none() {
        &Some("all".to_string())
    } else if !CONFIG.oshi.contains_key(oshi.as_ref().unwrap()) {
        &Some("invalid".to_string())
    } else {
        &oshi.clone().and_then(|alias| {
            CONFIG //
                .oshi
                .get(&alias)
                .cloned()
        })
    };

    let ctx = PageContext { channel_id: id.clone() };
    let content = Pages::Root.render(ctx).await.expect("failed to render page");

    headers.insert(CONTENT_TYPE, "text/plain".parse().unwrap());
    (StatusCode::OK, headers, content)
}

// GET /list
async fn get_list(
    TypedHeader(user_agent): TypedHeader<UserAgent>,
) -> impl axum::response::IntoResponse {
    let mut headers = HeaderMap::new();

    if !is_term(&user_agent) {
        headers.insert("Location", CONFIG.browser_redirect.parse().unwrap());
        return (StatusCode::TEMPORARY_REDIRECT, headers, "Redirecting to GitHub...".to_string());
    }

    let ctx = PageContext { channel_id: None };
    let content = Pages::List.render(ctx).await.expect("failed to render page");

    headers.insert(CONTENT_TYPE, "text/plain".parse().unwrap());
    (StatusCode::OK, headers, content)
}
