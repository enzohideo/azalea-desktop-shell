use azalea_service::StaticHandler;
use azalea_window::services;

#[tokio::main]
async fn main() {
    services::time::Service::listen(|out| {
        println!("hey there {out:?}");
        true
    })
    .join()
    .await;
}
