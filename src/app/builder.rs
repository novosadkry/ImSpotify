use super::App;
use rspotify::AuthCodeSpotify;

#[derive(Default)]
pub struct AppBuilder {
    cli: bool
}

impl AppBuilder {
    pub fn build(self) -> App {
        App {
            cli: self.cli,
            spotify: AuthCodeSpotify::default()
        }
    }

    pub fn cli(mut self) -> Self {
        self.cli = true; self
    }
}
