use super::App;
use super::IoEvent;

use std::{
    sync::Arc,
    time::Duration
};
use rspotify::model::PlayableItem;
use tokio::{
    sync::{
        Mutex,
        mpsc::Sender
    },
    time::Instant
};
use imgui::{
    Window,
    Dock,
    Ui,
    ProgressBar,
    im_str
};

fn init(io: &Sender<IoEvent>, app: &Arc<Mutex<App>>, run: &mut bool, ui: &mut Ui) {
    Dock::new().build(|root| {
        root.size(ui.io().display_size).split(
            imgui::Direction::Left,
            0.2_f32,
            |left| {
                left.dock_window(im_str!("Playlists"));
            },
            |right| {
                right.split(
                    imgui::Direction::Down,
                    0.2_f32,
                    |down| {
                        down.dock_window(im_str!("Playback"));
                    },
                    |up| {
                        up.dock_window(im_str!("Tracks"));
                        up.dock_window(im_str!("Properties"));
                    }
                );
            },
        )
    });

    io.blocking_send(IoEvent::FetchUserInfo).unwrap();
    io.blocking_send(IoEvent::FetchCurrentPlayback).unwrap();
}

pub fn main_loop(io: &Sender<IoEvent>, app: &Arc<Mutex<App>>, first_run: bool, run: &mut bool, ui: &mut Ui) {
    if first_run {
        init(io, app, run, ui);
    }

    let last_fetch = {
        let app = app.blocking_lock();
        app.spotify.state.last_fetch
            .unwrap_or(Instant::now())
            .elapsed().as_millis()
    };

    Window::new(im_str!("Playlists")).build(ui, || {});
    Window::new(im_str!("Tracks")).build(ui, || {});

    Window::new(im_str!("Properties")).build(ui, || {
        let app = app.blocking_lock();
        let state = &app.spotify.state;

        if let Some(me) = &state.me { {
            ui.text(format!(
                "Logged-in as: {}",
                me.display_name.to_owned().unwrap_or(String::new())
            ));
        };
    }});

    Window::new(im_str!("Playback")).build(ui, || {
        let app = app.blocking_lock();
        let state = &app.spotify.state;

        if let Some(playback) = &state.playback {
            let track = playback.item.to_owned().and_then(|i| {
                match i {
                    PlayableItem::Track(t) => {
                        Some((
                            t.name,
                            t.artists.iter()
                                .map(|a| a.name.clone())
                                .collect::<Vec<String>>(),
                            t.duration.as_millis()
                        ))
                    },
                    PlayableItem::Episode(e) => {
                        Some((e.name, vec![e.show.name], e.duration.as_millis()))
                    },
                }
            });

            if let Some(track) = track {
                let (name, artists, duration) = track;

                let mut progress = playback.progress
                    .unwrap_or(Duration::default())
                    .as_millis();

                if playback.is_playing {
                    progress += last_fetch;
                }

                ui.text(name);
                ui.text(artists.join(", "));
                ui.separator();
                ui.text(format!(
                    "{} / {}",
                    format_millis(progress),
                    format_millis(duration)
                ));

                ProgressBar::new(progress as f32 / duration as f32).build(ui);
            }
        }
    });

    if last_fetch > Duration::from_secs(5).as_millis() {
        io.blocking_send(IoEvent::FetchCurrentPlayback).unwrap();
    }

    *run = true;
}

fn format_millis(millis: u128) -> String {
    let minutes = millis / 60_000;
    let seconds = (millis % 60_000) / 1000;

    let seconds_display = if seconds < 10 {
        format!("0{}", seconds)
    } else {
        format!("{}", seconds)
    };

    if seconds == 60 {
        format!("{}:00", minutes + 1)
    } else {
        format!("{}:{}", minutes, seconds_display)
    }
}