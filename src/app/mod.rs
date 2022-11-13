pub use self::builder::AppBuilder;

use crate::{
    spotify::{
        Spotify,
        io::{Io, IoState, IoEvent, self},
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

#[derive(Clone)]
pub struct App {
    pub rt: Arc<Runtime>,
    pub cli: bool,
    pub spotify: Spotify
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn run(mut self) -> AppResult<()> {
        // Authenticate Spotify client
        self.spotify.client = self.rt.block_on(oauth_client())?;

        if !self.cli {
            // Initialize window system handler
            let system = system::init(file!());

            // Create state and channels for IO events
            let io_state = Arc::new(Mutex::new(IoState::default()));
            let (tx, rx) = mpsc::unbounded_channel();

            // Run the IO thread
            let io_handle = {
                let a = self.clone();
                let io = Io {
                    state: io_state.clone(),
                    receiver: Some(rx),
                    sender: None
                };

                self.rt.spawn(async move {
                    io::main_loop(io, a).await;
                })
            };

            // Run the UI thread
            {
                let a = self.clone();
                let io = Io {
                    state: io_state,
                    receiver: None,
                    sender: Some(tx)
                };

                system.main_loop(move |s, r, u| {
                    ui::main_loop(&io, &a, s, r, u);
                });
            }

            // Gracefully exit the IO thread
            self.rt.block_on(io_handle)?;
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
