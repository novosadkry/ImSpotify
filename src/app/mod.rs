pub use self::builder::AppBuilder;
use crate::System;

use rspotify::{prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth};

mod builder;

pub struct App {
    pub system: Option<System>
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub async fn run(self) {
        if let Some(system) = self.system {
            system.main_loop(move |run, ui| ui.show_demo_window(run));
        }

        else {
            let oauth = OAuth {
                redirect_uri: String::from("http://localhost:8888/callback"),
                scopes: scopes!("user-library-read"),
                ..Default::default()
            };
            let creds = Credentials::from_env().unwrap();

            let mut spotify = AuthCodeSpotify::new(creds, oauth);

            let url = spotify.get_authorize_url(false).unwrap();
            spotify.prompt_for_token(&url).await.unwrap();

            let page = spotify.current_user_saved_tracks_manual(None, None, None)
                .await
                .unwrap();

            println!("Items:");
            for item in page.items {
                println!("* {}", item.track.name);
            }
        }
    }
}

