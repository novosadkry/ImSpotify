pub use self::builder::AppBuilder;

use crate::{spotify, system};

use std::sync::Arc;
use anyhow::{Context, Result};
use rspotify::AuthCodeSpotify;
use rspotify::prelude::OAuthClient;

mod builder;
mod ui;

pub type AppResult<T> = Result<T>;

pub struct App {
    pub cli: bool,
    pub spotify: AuthCodeSpotify
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub async fn run(mut self) -> AppResult<()> {
        self.spotify = spotify::auth::oauth_client().await?;

        if !self.cli {
            let system = system::init(file!());
            let app = Arc::new(self);

            system.main_loop(move |f, r, u| {
                ui::main_loop(&app, f, r, u);
            });
        }

        else {
            let user = self.spotify.me().await?;
            println!("Logged-in as: {}", user.display_name.unwrap());

            let cmd = std::env::args()
                .nth(1)
                .context("Invalid argument")?;

            self.handle_command(cmd.as_str()).await?;
        }

        Ok(())
    }

    async fn handle_command(&self, cmd: &str) -> AppResult<()> {
        match cmd {
            "--resume" => {
                self.spotify.resume_playback(None, None).await
                    .context("Unable to resume playback")?;
            }
            "--pause" => {
                self.spotify.pause_playback(None).await
                    .context("Unable to pause playback")?
            }
            _ => todo!()
        };

        Ok(())
    }
}

