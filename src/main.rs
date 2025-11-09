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
use config::CONFIG;
use dotenv::dotenv;
use pages::{PageContext, Pages, Render};
use reqwest::{
    Method,
    header::{ACCEPT, USER_AGENT},
};
use tower_http::cors;
use utils::is_term;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt().with_max_level(CONFIG.server.log_level).init();

    sqlite::init_db();
    scheduler::init_scheduler().await.expect("failed to init scheduler");

    let cors = cors::CorsLayer::new() //
        .allow_methods([Method::GET])
        .allow_origin(cors::Any);

    let router = Router::new() //
        .fallback(Redirect::temporary(&CONFIG.browser_redirect)) // This should handle term too
        .route("/", get(get_root))
        .route("/list", get(get_list))
        .layer(cors);

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
    req_headers: HeaderMap,
) -> impl axum::response::IntoResponse {
    let mut res_headers = HeaderMap::new();

    let user_agent = req_headers.get(USER_AGENT);
    let return_json = match req_headers.get(ACCEPT) {
        Some(val) => match val.to_str() {
            Ok(s) => s.contains("application/json"),
            Err(_) => false,
        },
        None => false,
    };

    if !is_term(user_agent) && !return_json {
        res_headers.insert("Location", CONFIG.browser_redirect.parse().unwrap());
        return (
            StatusCode::TEMPORARY_REDIRECT,
            res_headers,
            "Redirecting to GitHub...".to_string(),
        );
    }

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

    if return_json {
        let content = Pages::Root.render_json(ctx).await.expect("failed to render json page");

        res_headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        return (StatusCode::OK, res_headers, content);
    }

    let content = Pages::Root.render_text(ctx).await.expect("failed to render text page");

    res_headers.insert(CONTENT_TYPE, "text/plain".parse().unwrap());
    (StatusCode::OK, res_headers, content)
}

// GET /list
async fn get_list(req_headers: HeaderMap) -> impl axum::response::IntoResponse {
    let mut res_headers = HeaderMap::new();
    let user_agent = req_headers.get(USER_AGENT);
    let return_json = match req_headers.get(ACCEPT) {
        Some(val) => match val.to_str() {
            Ok(s) => s.contains("application/json"),
            Err(_) => false,
        },
        None => false,
    };

    if !is_term(user_agent) && !return_json {
        res_headers.insert("Location", CONFIG.browser_redirect.parse().unwrap());
        return (
            StatusCode::TEMPORARY_REDIRECT,
            res_headers,
            "Redirecting to GitHub...".to_string(),
        );
    }

    let ctx = PageContext { channel_id: None };

    if return_json {
        let content = Pages::Root.render_json(ctx).await.expect("failed to render json page");

        res_headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
        return (StatusCode::OK, res_headers, content);
    }

    let content = Pages::List.render_text(ctx).await.expect("failed to render text page");

    res_headers.insert(CONTENT_TYPE, "text/plain".parse().unwrap());
    (StatusCode::OK, res_headers, content)
}
