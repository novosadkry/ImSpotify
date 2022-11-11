use crate::{App, AppResult};

use std::sync::Arc;
use rspotify::{
    prelude::OAuthClient,
    model::AdditionalType
};
use tokio::{
    sync::{
        Mutex,
        mpsc::Receiver
    },
    time::Instant
};

#[derive(Debug)]
pub enum IoEvent {
    FetchUserInfo,
    FetchCurrentPlayback
}

pub async fn main_loop(io: &mut Receiver<IoEvent>, app: &Arc<Mutex<App>>) {
    while let Some(event) = io.recv().await {
        match handle_event(event, app).await {
            Ok(_) => continue,
            Err(e) => eprintln!("Error in IO thread: {}", e),
        };
    }
}

pub async fn handle_event(event: IoEvent, app: &Arc<Mutex<App>>) -> AppResult<()> {
    let mut app = app.lock().await;
    let spotify = &mut app.spotify;

    let client = &spotify.client;
    let state = &mut spotify.state;

    match event {
        IoEvent::FetchUserInfo => {
            state.me = Some(client.me().await?);
        },
        IoEvent::FetchCurrentPlayback => {
            let playback = client.current_playback(
                None,
                Some(vec![&AdditionalType::Episode, &AdditionalType::Track])
            ).await?;

            state.playback = playback;
            state.last_fetch = Some(Instant::now());
        }
    };

    Ok(())
}