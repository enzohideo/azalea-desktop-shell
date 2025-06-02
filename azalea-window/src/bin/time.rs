use azalea_service::StaticHandler;
use azalea_window::service;

#[tokio::main]
async fn main() {
    service::time::Service::listen(|out| {
        println!("hey there {out:?}");
        true
    })
    .join()
    .await;
}
