use rspotify::prelude::BaseClient;

mod auth;

#[tokio::main]

async fn main() -> anyhow::Result<()> {
    let cache =
        librespot_core::cache::Cache::new(Some(std::path::Path::new("/tmp")), None, None, None)?;
    let token = auth::get_token(cache).await?;
    let client = rspotify::AuthCodeSpotify::from_token(token);

    let id = rspotify::model::TrackId::from_id("1RMJOxR6GRPsBHL8qeC2ux")?;
    let track = client.track(id, None).await?;
    println!("track: {track:?}");

    Ok(())
}
