use super::App;
use crate::system;

use rspotify::AuthCodeSpotify;

#[derive(Default)]
pub struct AppBuilder {
    cli: bool
}

impl AppBuilder {
    pub fn build(self) -> App {
        let system = if !self.cli {
            Some(system::init(file!()))
        } else { None };

        App {
            system: system,
            spotify: AuthCodeSpotify::default()
        }
    }

    pub fn cli(mut self) -> Self {
        self.cli = true; self
    }
}
