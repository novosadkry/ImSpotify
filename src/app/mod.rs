pub use self::builder::AppBuilder;

use crate::{
    spotify::{
        Spotify,
        io::{IoEvent, self},
        auth::oauth_client
    },
    system
};

use std::sync::Arc;
use tokio::{
    runtime::Runtime,
    sync::{Mutex, mpsc}
};
use anyhow::{Context, Result};
use rspotify::prelude::OAuthClient;

mod builder;
mod ui;

pub type AppResult<T> = Result<T>;

pub struct App {
    pub rt: Runtime,
    pub cli: bool,
    pub spotify: Spotify
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn run(mut self) -> AppResult<()> {
        // Authenticate Spotify client
        self.spotify = Spotify {
            client: self.rt.block_on(oauth_client())?,
            ..Default::default()
        };

        if !self.cli {
            // Initialize window system handler
            let system = system::init(file!());

            // Make two thread-safe references to self
            let app = Arc::new(Mutex::new(self));

            // Create channel for IO events
            let (tx, mut rx) = mpsc::unbounded_channel();

            // Run the IO thread
            let io_handle = {
                let a = app.clone();
                app.blocking_lock().rt.spawn(async move {
                    io::main_loop(&mut rx, &a).await;
                })
            };

            // Run the UI thread
            {
                let a = app.clone();
                system.main_loop(move |f, r, u| {
                    ui::main_loop(&tx, &a, f, r, u);
                });
            }

            // Gracefully exit the IO thread
            app.blocking_lock().rt.block_on(io_handle)?;
        }

        else {
            let user = self.rt.block_on(self.spotify.client.me())?;
            println!("Logged-in as: {}", user.display_name.unwrap_or(String::new()));

            let cmd = std::env::args()
                .nth(1)
                .context("Invalid argument")?;

            // Parse and handle commands
            self.rt.block_on(self.handle_command(cmd.as_str()))?;
        }

        Ok(())
    }

    async fn handle_command(&self, cmd: &str) -> AppResult<()> {
        let client = &self.spotify.client;

        match cmd {
            "--resume" => {
                client.resume_playback(None, None).await
                    .context("Unable to resume playback")?;
            }
            "--pause" => {
                client.pause_playback(None).await
                    .context("Unable to pause playback")?
            }
            _ => todo!()
        };

        Ok(())
    }
}
