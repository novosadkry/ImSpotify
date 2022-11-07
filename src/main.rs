use im_spotify::{App, AppResult};

#[tokio::main]
async fn main() -> AppResult<()> {
    App::builder()
        .cli().build()
        .run().await?;

    Ok(())
}
