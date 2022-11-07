pub use self::builder::AppBuilder;

use crate::System;
use crate::spotify;

use std::error::Error;
use rspotify::prelude::OAuthClient;

mod builder;

pub type AppResult<T> = Result<T, Box<dyn Error>>;

pub struct App {
    pub system: Option<System>
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub async fn run(self) -> AppResult<()> {
        if let Some(system) = self.system {
            system.main_loop(move |run, ui| ui.show_demo_window(run));
        }

        else {
            let client = spotify::auth::oauth_client().await?;
            let cmd = std::env::args()
                .nth(1)
                .ok_or("Invalid argument")?;

            if cmd == "--resume" {
                client.resume_playback(None, None).await?;
            }

            if cmd == "--pause" {
                client.pause_playback(None).await?;
            }
        }

        Ok(())
    }
}

