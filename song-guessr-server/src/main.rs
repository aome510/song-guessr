mod api;
mod client;
mod game;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = client::Client::new();
    client.get_token().await?;

    let app = api::new_app(client);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
