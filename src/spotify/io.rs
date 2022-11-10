use crate::{App, AppResult};

use std::sync::Arc;
use rspotify::prelude::{
    BaseClient,
    OAuthClient
};
use tokio::sync::{
    Mutex,
    mpsc::Receiver
};

#[derive(Debug)]
pub enum IoEvent {
    FetchUserInfo
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

    match event {
        IoEvent::FetchUserInfo => {
            spotify.me = Some(client.me().await?);
        }
    };

    Ok(())
}