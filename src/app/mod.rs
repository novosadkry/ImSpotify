pub use self::builder::AppBuilder;
use crate::System;

mod builder;

pub struct App {
    pub system: Option<System>
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn run(self) {
        if let Some(system) = self.system {
            system.main_loop(move |run, ui| ui.show_demo_window(run));
        }

        else {
            println!("ImSpotify cli")
        }
    }
}

