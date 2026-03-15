mod api;
mod colors;
mod config;
mod pages;
mod scheduler;
mod sqlite;
mod time;
mod utils;
mod youtube;

use anyhow::Result;
use axum::{
    Router,
    extract::Query,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::Redirect,
    routing::get,
};
use config::CONFIG;
use dotenvy::dotenv;
use pages::{PageContext, Pages, Render};
use reqwest::{
    Method,
    header::{ACCEPT, USER_AGENT},
};
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};
use tokio::net::TcpListener;
use tower_http::cors;
use utils::is_term;

enum ResponseFormat {
    Text,
    Json,
    Browser,
}

fn negotiate(headers: &HeaderMap) -> ResponseFormat {
    let user_agent = headers.get(USER_AGENT);

    if let Some(val) = headers.get(ACCEPT)
        && let Ok(s) = val.to_str()
    {
        if s.contains("application/json") {
            return ResponseFormat::Json;
        }
        if s.contains("text/plain") || is_term(user_agent) {
            return ResponseFormat::Text;
        }
    }

    if is_term(user_agent) {
        return ResponseFormat::Text;
    }

    ResponseFormat::Browser
}

const CACHE_CONTROL_VALUE: HeaderValue = HeaderValue::from_static("public, max-age=60");
const JSON_HEADER: HeaderValue = HeaderValue::from_static("application/json");
const TEXT_HEADER: HeaderValue = HeaderValue::from_static("text/plain");

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt().with_max_level(CONFIG.server.log_level).init();

    sqlite::init_db();
    scheduler::init_scheduler().await.expect("failed to init scheduler");

    let cors = cors::CorsLayer::new() //
        .allow_methods([Method::GET])
        .allow_origin(cors::Any);

    let router = Router::new()
        .fallback(Redirect::temporary(&CONFIG.browser_redirect))
        .route("/health", get(get_health))
        .route("/", get(get_root))
        .route("/list", get(get_list))
        .layer(cors);

    let host = Ipv4Addr::from_str(&CONFIG.server.host).expect("invalid host");
    let socket = SocketAddr::from((host, CONFIG.server.port));
    let listener = TcpListener::bind(&socket).await?;

    tracing::info!("listening on http://{}", &socket);

    axum::serve(
        listener, //
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

// GET /health
async fn get_health() -> StatusCode {
    StatusCode::OK
}

// GET /
async fn get_root(
    query: Query<HashMap<String, String>>,
    req_headers: HeaderMap,
) -> impl axum::response::IntoResponse {
    let mut res_headers = HeaderMap::new();

    match negotiate(&req_headers) {
        ResponseFormat::Browser => {
            res_headers.insert(header::LOCATION, CONFIG.browser_redirect.parse().unwrap());
            (StatusCode::TEMPORARY_REDIRECT, res_headers, "Redirecting to GitHub...".to_string())
        }
        format => {
            let oshi = query.get("oshi").cloned();
            let id: &Option<String> = match &oshi {
                None => &Some("all".to_string()),
                Some(alias) if !CONFIG.oshi.contains_key(alias) => &Some("invalid".to_string()),
                Some(_) => &oshi.clone().and_then(|alias| {
                    CONFIG //
                        .oshi
                        .get(&alias)
                        .cloned()
                }),
            };

            let ctx = PageContext { channel_id: id.clone() };
            res_headers.insert(header::CACHE_CONTROL, CACHE_CONTROL_VALUE);

            match format {
                ResponseFormat::Json => {
                    let content = Pages::Root //
                        .render_json(ctx)
                        .await
                        .expect("failed to render json page");

                    res_headers.insert(header::CONTENT_TYPE, JSON_HEADER);

                    (StatusCode::OK, res_headers, content)
                }
                _ => {
                    let content = Pages::Root //
                        .render_text(ctx)
                        .await
                        .expect("failed to render text page");

                    res_headers.insert(header::CONTENT_TYPE, TEXT_HEADER);

                    (StatusCode::OK, res_headers, content)
                }
            }
        }
    }
}

// GET /list
async fn get_list(req_headers: HeaderMap) -> impl axum::response::IntoResponse {
    let mut res_headers = HeaderMap::new();

    match negotiate(&req_headers) {
        ResponseFormat::Browser => {
            res_headers.insert(header::LOCATION, CONFIG.browser_redirect.parse().unwrap());
            (StatusCode::TEMPORARY_REDIRECT, res_headers, "Redirecting to GitHub...".to_string())
        }
        format => {
            let ctx = PageContext { channel_id: None };
            res_headers.insert(header::CACHE_CONTROL, CACHE_CONTROL_VALUE);

            match format {
                ResponseFormat::Json => {
                    let content = Pages::List //
                        .render_json(ctx)
                        .await
                        .expect("failed to render json page");

                    res_headers.insert(header::CONTENT_TYPE, JSON_HEADER);

                    (StatusCode::OK, res_headers, content)
                }
                _ => {
                    let content = Pages::Root //
                        .render_text(ctx)
                        .await
                        .expect("failed to render text page");

                    res_headers.insert(header::CONTENT_TYPE, TEXT_HEADER);

                    (StatusCode::OK, res_headers, content)
                }
            }
        }
    }
}
