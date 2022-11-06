use super::App;
use crate::system;

#[derive(Default)]
pub struct AppBuilder {
    cli: bool
}

impl AppBuilder {
    pub fn build(self) -> App {
        let system = if !self.cli {
            Some(system::init(file!()))
        } else { None };

        App {
            system
        }
    }

    pub fn cli(mut self) -> Self {
        self.cli = true; self
    }
}
