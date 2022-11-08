pub use self::builder::AppBuilder;

use crate::System;
use crate::spotify;

use std::error::Error;
use rspotify::AuthCodeSpotify;
use rspotify::prelude::OAuthClient;

mod builder;

pub type AppResult<T> = Result<T, Box<dyn Error>>;

pub struct App {
    pub system: Option<System>,
    pub spotify: AuthCodeSpotify
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub async fn run(mut self) -> AppResult<()> {
        self.spotify = spotify::auth::oauth_client().await?;

        if let Some(system) = self.system {
            system.main_loop(move |run, ui| ui.show_demo_window(run));
        }

        else {
            let user = self.spotify.me().await?;
            println!("Logged-in as: {}", user.display_name.unwrap());

            let cmd = std::env::args()
                .nth(1)
                .ok_or("Invalid argument")?;

            self.handle_command(cmd.as_str()).await?;
        }

        Ok(())
    }

    async fn handle_command(&self, cmd: &str) -> AppResult<()> {
        match cmd {
            "--resume" => {
                self.spotify.resume_playback(None, None).await
                    .or(Err("Unable to resume playback"))?;
            }
            "--pause" => {
                self.spotify.pause_playback(None).await
                    .or(Err("Unable to pause playback"))?
            }
            _ => todo!()
        };

        Ok(())
    }
}

