use super::App;

#[derive(Default)]
pub struct AppBuilder {
    cli: bool
}

impl AppBuilder {
    pub fn build(self) -> App {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();

        App {
            rt: runtime,
            cli: self.cli,
            spotify: Default::default()
        }
    }

    pub fn cli(mut self) -> Self {
        self.cli = true; self
    }
}
