#[tokio::main]
async fn main() {
    let client = system_tray::client::Client::new().await.unwrap();
    let mut tray_rx = client.subscribe();

    let initial_items = client.items();
    let items = initial_items.lock().unwrap();
    for (stuff, item) in items.clone().into_iter() {
        println!("{}", stuff);
        println!("{:?}", item);
    }

    // TODO: Fixme: figure out why this is 0
    println!("{}", items.len());

    while let Ok(ev) = tray_rx.recv().await {
        println!("{ev:?}");
    }
}
