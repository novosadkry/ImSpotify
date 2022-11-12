use super::App;

use std::sync::Arc;

#[derive(Default)]
pub struct AppBuilder {
    cli: bool
}

impl AppBuilder {
    pub fn build(self) -> App {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        App {
            rt: Arc::new(runtime),
            cli: self.cli,
            spotify: Default::default()
        }
    }

    pub fn cli(mut self) -> Self {
        self.cli = true; self
    }
}
