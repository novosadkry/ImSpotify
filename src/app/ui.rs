use super::App;

use std::sync::Arc;
use imgui::{
    Window,
    Dock,
    Ui,
    im_str
};

pub fn main_loop(app: &Arc<App>, first_run: bool, run: &mut bool, ui: &mut Ui) {
    Window::new(im_str!("Viewport")).build(ui, || {});
    Window::new(im_str!("NodeGraph")).build(ui, || {});
    Window::new(im_str!("Properties")).build(ui, || {});
    Window::new(im_str!("Outliner")).build(ui, || {});

    if first_run
    {
        Dock::new().build(|root| {
            root.size(ui.io().display_size).split(
                imgui::Direction::Left,
                0.7_f32,
                |left| {
                    left.dock_window(im_str!("Viewport"));
                    left.dock_window(im_str!("NodeGraph"));
                },
                |right| {
                    right.dock_window(im_str!("Properties"));
                    right.dock_window(im_str!("Outliner"));
                },
            )
        });
    }

    *run = true;
}