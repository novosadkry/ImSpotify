use super::App;
use super::IoEvent;

use std::sync::Arc;
use tokio::sync::{
    Mutex,
    mpsc::Sender
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
                        up.dock_window(im_str!("Properties"));
                    }
                );
            },
        )
    });

    io.blocking_send(IoEvent::FetchUserInfo).unwrap();
}

pub fn main_loop(io: &Sender<IoEvent>, app: &Arc<Mutex<App>>, first_run: bool, run: &mut bool, ui: &mut Ui) {
    if first_run {
        init(io, app, run, ui);
    }

    Window::new(im_str!("Playlists")).build(ui, || {});

    Window::new(im_str!("Properties")).build(ui, || {
        match &app.blocking_lock().spotify.me {
            Some(me) => ui.text(format!(
                "Logged-in as: {}",
                me.display_name.as_ref().unwrap_or(&String::new())
            )),
            None => ui.text("Unloaded")
        }
    });

    Window::new(im_str!("Playback"))
        .build(ui, || {
            ui.text("This is a simple progress bar:");
            ProgressBar::new(0.5).build(ui);
        });

    *run = true;
}