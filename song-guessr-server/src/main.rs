use anyhow::Context;
use axum::{
    http::{header::CONTENT_TYPE, HeaderValue, Method},
    routing::get,
    Router,
};
use rspotify::prelude::BaseClient;
use tower_http::cors::CorsLayer;

mod auth;

async fn _get() -> anyhow::Result<String> {
    let cache =
        librespot_core::cache::Cache::new(Some(std::path::Path::new("/tmp")), None, None, None)?;
    let token = auth::get_token(cache).await?;
    let client = rspotify::AuthCodeSpotify::from_token(token);

    let id = rspotify::model::TrackId::from_id("1RMJOxR6GRPsBHL8qeC2ux")?;
    let track = client.track(id, None).await?;
    Ok(track.preview_url.context("no preview url")?)
}

async fn get_api() -> Result<String, String> {
    _get().await.map_err(|e| e.to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE]);

    let app = Router::new().route("/get", get(get_api)).layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
