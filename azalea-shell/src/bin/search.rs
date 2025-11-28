use azalea_service::StaticHandler;
use azalea_shell::service;

#[tokio::main]
async fn main() {
    service::search::Service::init(service::search::Init {});

    service::search::Service::listen(|out| {
        azalea_log::message!("Search output received:\n{out:#?}");
        true
    })
    .join()
    .await;
}
