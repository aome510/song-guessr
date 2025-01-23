mod api;
mod client;
mod game;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = api::new_app();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
