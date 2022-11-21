use crate::System;
use crate::spotify::io::Io;

use super::App;
use super::IoEvent;

use std::time::Duration;
use rspotify::model::{
    PlayableItem,
    SimplifiedPlaylist
};
use tokio::time::Instant;
use imgui::{
    Window,
    Ui,
    DockNode,
    ProgressBar,
    Selectable,
    StyleColor,
    ImString,
    im_str,
    sys::{
        self,
        ImGuiDockNodeFlags_NoCloseButton,
        ImGuiDockNodeFlags_AutoHideTabBar
    }
};

pub fn main_loop(io: &Io, app: &App, system: &System, run: &mut bool, ui: &mut Ui) {
    let dock_id = draw_dock();

    if system.first_run {
        dock_layout(dock_id, ui);
        fetch_init_state(io);
    }

    draw_playlists(io, app, ui);
    draw_tracks(io, app, ui);
    draw_properties(app, ui);
    draw_playback(io, app, ui);

    *run = true;
}

fn dock_layout(dock_id: u32, ui: &mut Ui) {
    let root = DockNode::new(dock_id);

    root.size(ui.io().display_size).split(
        imgui::Direction::Left,
        0.2_f32,
        |left| {
            left.split(
                imgui::Direction::Down,
                0.05_f32,
                |down| {
                    down.dock_window(im_str!("Properties"));
                },
                |up| {
                    up.dock_window(im_str!("Playlists"));
                }
            );
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
                }
            );
        },
    );
}

fn fetch_init_state(io: &Io) {
    let sender = io.sender.as_ref().unwrap();
    sender.send(IoEvent::FetchUserInfo).unwrap();
    sender.send(IoEvent::FetchPlaylists).unwrap();
    sender.send(IoEvent::FetchCurrentPlayback).unwrap();
}

fn draw_dock() -> u32 {
    unsafe {
        sys::igDockSpaceOverViewport(
            sys::igGetMainViewport(),
            ImGuiDockNodeFlags_NoCloseButton as i32 |
            ImGuiDockNodeFlags_AutoHideTabBar as i32,
            ::std::ptr::null::<sys::ImGuiWindowClass>(),
        )
    }
}

fn draw_playlists(io: &Io, app: &App, ui: &mut Ui) {
    Window::new(im_str!("Playlists")).build(ui, || {
        let sender = io.sender.as_ref().unwrap();
        let mut app_state = app.spotify.state.blocking_lock();

        let mut selected_playlist: Option<SimplifiedPlaylist> = None;

        if let Some(playlists) = &app_state.playlists {
            for playlist in playlists {
                let mut selected: bool = false;

                let stack = {
                    app_state.selected_playlist.as_ref().and_then(|p| {
                        if p.id == playlist.id {
                            Some(ui.push_style_color(StyleColor::Text, [0.7, 1.0, 1.0, 1.0]))
                        } else {
                            None
                        }
                    })
                };

                Selectable::new(&ImString::new(&playlist.name)[..])
                    .build_with_ref(ui, &mut selected);

                if let Some(stack) = stack {
                    stack.pop(ui);
                }

                if selected {
                    let playlist_id = playlist.id.clone();
                    selected_playlist = Some(playlist.clone());
                    sender.send(IoEvent::FetchPlaylistItems(playlist_id)).unwrap();
                }
            }
        }

        if let Some(selected) = selected_playlist {
            app_state.selected_playlist = Some(selected);
        }
    });
}

fn draw_tracks(io: &Io, app: &App, ui: &mut Ui) {
    Window::new(im_str!("Tracks")).build(ui, || {
        let sender = io.sender.as_ref().unwrap();
        let app_state = app.spotify.state.blocking_lock();

        if let Some(items) = &app_state.selected_playlist_items {
            let items = items
                .iter()
                .filter(|t| !t.is_local)
                .map(|t| &t.track);

            for item in items {
                if let Some(PlayableItem::Track(track)) = item {
                    let mut selected: bool = false;
                    let stack = ui.push_style_color(StyleColor::Text, [0.7, 0.7, 0.7, 1.0]);

                    let artists = track.artists
                        .iter()
                        .map(|a| a.name.clone())
                        .collect::<Vec<String>>()
                        .join(", ");

                    Selectable::new(&ImString::new(&artists)[..])
                        .build_with_ref(ui, &mut selected);

                    stack.pop(ui);

                    let stack = {
                        app_state.playback.as_ref().and_then(|playback| {
                            if let Some(PlayableItem::Track(playback_track)) = &playback.item {
                                if track.id.as_ref().unwrap() == playback_track.id.as_ref().unwrap() {
                                    Some(ui.push_style_color(StyleColor::Text, [0.7, 1.0, 1.0, 1.0]))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                    };

                    let x = ui.cursor_pos()[0];
                    ui.same_line(x + 250.0);
                    ui.text(ImString::new(&track.name));

                    if let Some(stack) = stack {
                        stack.pop(ui);
                    }

                    if selected {
                        let track_id = track.id.clone().unwrap();
                        sender.send(IoEvent::PushPlayback(track_id)).unwrap();
                    }
                }
            }
        }
    });
}

fn draw_properties(app: &App, ui: &mut Ui) {
    Window::new(im_str!("Properties")).build(ui, || {
        let app_state = app.spotify.state.blocking_lock();

        if let Some(me) = &app_state.me {
            ui.text(format!(
                "Logged-in as: {}",
                me.display_name.to_owned().unwrap_or(String::new())
            ));
        };
    });
}

fn draw_playback(io: &Io, app: &App, ui: &mut Ui) {
    Window::new(im_str!("Playback")).build(ui, || {
        let app_state = app.spotify.state.blocking_lock();

        if let Some(playback) = &app_state.playback {
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

                let last_fetch = {
                    let io_state = io.state.blocking_lock();
                    io_state.playback_last_fetch
                        .unwrap_or(Instant::now())
                        .elapsed().as_millis()
                };

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