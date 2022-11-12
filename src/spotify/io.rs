use crate::{App, AppResult};

use std::{sync::Arc, time::Duration};
use rspotify::{
    prelude::OAuthClient,
    model::AdditionalType
};
use tokio::{
    sync::{
        Mutex,
        mpsc::{
            UnboundedSender,
            UnboundedReceiver
        }
    },
    time::Instant
};

#[derive(Default)]
pub struct Io {
    pub state: Arc<Mutex<IoState>>,
    pub sender: Option<UnboundedSender<IoEvent>>,
    pub receiver: Option<UnboundedReceiver<IoEvent>>
}

#[derive(Default)]
pub struct IoState {
    pub playback_last_fetch: Option<Instant>
}

#[derive(Debug)]
pub enum IoEvent {
    FetchUserInfo,
    FetchCurrentPlayback
}

impl Clone for Io {
    fn clone(&self) -> Self {
        Io {
            state: self.state.clone(),
            ..Default::default()
        }
    }
}

pub async fn main_loop(mut io: Io, app: App) {
    let mut receiver = io.receiver
        .take().unwrap();

    let playback_task = {
        let io = io.clone();
        let app = app.clone();

        tokio::spawn(async move { loop {
            let elapsed = {
                let io_state = io.state.lock().await;
                io_state.playback_last_fetch
                    .and_then(|i| Some(i.elapsed().as_millis() > Duration::from_secs(5).as_millis()))
                    .unwrap_or(false)
            };

            if elapsed {
                match handle_event(IoEvent::FetchCurrentPlayback, &io, &app).await {
                    Ok(_) => continue,
                    Err(e) => eprintln!("Error in IO thread: {}", e),
                };
            }
        }})
    };

    while let Some(event) = receiver.recv().await {
        match handle_event(event, &io, &app).await {
            Ok(_) => continue,
            Err(e) => eprintln!("Error in IO thread: {}", e),
        };
    }

    playback_task.await.unwrap();
}

pub async fn handle_event(event: IoEvent, io: &Io, app: &App) -> AppResult<()> {
    let client = &app.spotify.client;

    match event {
        IoEvent::FetchUserInfo => {
            let me = client.me().await?;
            let app_state = &mut app
                .spotify.state
                .lock().await;

            app_state.me = Some(me);
        },

        IoEvent::FetchCurrentPlayback => {
            let playback = client.current_playback(
                None,
                Some(vec![&AdditionalType::Episode, &AdditionalType::Track])
            ).await?;

            let app_state = &mut app.spotify.state
                .lock().await;

            let io_state = &mut io.state
                .lock().await;

            app_state.playback = playback;
            io_state.playback_last_fetch = Some(Instant::now());
        }
    };

    Ok(())
}