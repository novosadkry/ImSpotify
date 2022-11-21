pub mod auth;
pub mod io;

use std::sync::Arc;
use tokio::sync::Mutex;
use rspotify::{
    AuthCodeSpotify,
    model::{
        PrivateUser,
        CurrentPlaybackContext,
        SimplifiedPlaylist, PlaylistItem
    }
};

#[derive(Clone)]
pub struct Spotify {
    pub client: AuthCodeSpotify,
    pub state: Arc<Mutex<SpotifyState>>
}

#[derive(Default)]
pub struct SpotifyState {
    pub me: Option<PrivateUser>,
    pub playback: Option<CurrentPlaybackContext>,
    pub playlists: Option<Vec<SimplifiedPlaylist>>,
    pub selected_playlist: Option<SimplifiedPlaylist>,
    pub selected_playlist_items: Option<Vec<PlaylistItem>>
}

impl Default for Spotify {
    fn default() -> Self {
        Self {
            client: Default::default(),
            state: Arc::new(Mutex::new(Default::default()))
        }
    }
}
