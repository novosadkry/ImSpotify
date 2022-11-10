use imspotify::{App, AppResult};

fn main() -> AppResult<()> {
    App::builder()
        .build()
        .run()?;

    Ok(())
}
