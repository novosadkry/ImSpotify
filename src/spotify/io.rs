use crate::{App, AppResult};

use std::{sync::Arc, time::Duration};
use rspotify::{
    prelude::OAuthClient,
    model::AdditionalType
};
use tokio::{
    sync::{
        Mutex,
        mpsc::UnboundedReceiver
    },
    time::Instant
};

#[derive(Debug)]
pub enum IoEvent {
    FetchUserInfo,
    FetchCurrentPlayback
}

pub async fn main_loop(io: &mut UnboundedReceiver<IoEvent>, app: &Arc<Mutex<App>>) {
    let app_clone = app.clone();

    let playback_task = tokio::spawn(async move { loop {
        let elapsed = {
            let app = app_clone.lock().await;
            app.spotify.state.last_fetch
                .and_then(|i| Some(i.elapsed().as_millis() > Duration::from_secs(5).as_millis()))
                .unwrap_or(false)
        };

        if elapsed {
            match handle_event(IoEvent::FetchCurrentPlayback, &app_clone).await {
                Ok(_) => continue,
                Err(e) => eprintln!("Error in IO thread: {}", e),
            };
        }
    }});

    while let Some(event) = io.recv().await {
        match handle_event(event, app).await {
            Ok(_) => continue,
            Err(e) => eprintln!("Error in IO thread: {}", e),
        };
    }

    playback_task.await.unwrap();
}

pub async fn handle_event(event: IoEvent, app: &Arc<Mutex<App>>) -> AppResult<()> {
    match event {
        IoEvent::FetchUserInfo => {
            let me = {
                let client = app
                    .lock().await
                    .spotify.client
                    .to_owned();

                async move {
                    client.me().await
                }
            };

            let me = me.await?;
            let state = &mut app
                .lock().await
                .spotify.state;

            state.me = Some(me);
        },

        IoEvent::FetchCurrentPlayback => {
            let playback = {
                let client = app
                    .lock().await
                    .spotify.client
                    .to_owned();

                async move {
                    client.current_playback(
                        None,
                        Some(vec![&AdditionalType::Episode, &AdditionalType::Track])
                    ).await
                }
            };

            let playback = playback.await?;
            let state = &mut app
                .lock().await
                .spotify.state;

            state.playback = playback;
            state.last_fetch = Some(Instant::now());
        }
    };

    Ok(())
}