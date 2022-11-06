use im_spotify::App;

#[tokio::main]
async fn main() {
    App::builder()
        .cli().build()
        .run().await;
}
