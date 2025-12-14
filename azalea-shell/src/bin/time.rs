use azalea_service::StaticServiceManager;
use azalea_shell::service;

#[tokio::main]
async fn main() {
    service::time::Service::listen(|out| {
        println!("hey there {out:?}");
        true
    })
    .join()
    .await;
}
