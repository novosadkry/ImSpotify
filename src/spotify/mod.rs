pub mod auth;
pub mod io;

use tokio::time::Instant;
use rspotify::{
    AuthCodeSpotify,
    model::{
        PrivateUser,
        CurrentPlaybackContext
    }
};

#[derive(Default)]
pub struct Spotify {
    pub client: AuthCodeSpotify,
    pub state: State
}

#[derive(Default)]
pub struct State {
    pub me: Option<PrivateUser>,
    pub playback: Option<CurrentPlaybackContext>,
    pub last_fetch: Option<Instant>
}
