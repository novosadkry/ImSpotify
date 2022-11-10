pub mod auth;
pub mod io;

use rspotify::{
    AuthCodeSpotify,
    model::PrivateUser
};

#[derive(Default)]
pub struct Spotify {
    pub client: AuthCodeSpotify,
    pub me: Option<PrivateUser>
}
