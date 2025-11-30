use wayland_client::{Connection, Dispatch, QueueHandle, protocol::wl_registry};
use wayland_protocols::ext::workspace::v1::client::ext_workspace_manager_v1::ExtWorkspaceManagerV1;
struct AppData;

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        _state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("{}", interface);
            if interface == "ext_workspace_manager_v1" {
                println!("CREATED {}", interface);
                let _proxy: ExtWorkspaceManagerV1 =
                    registry.bind::<ExtWorkspaceManagerV1, _, _>(name, version, qh, ());
            }
        }
    }
}

impl Dispatch<ExtWorkspaceManagerV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &ExtWorkspaceManagerV1,
        event: <ExtWorkspaceManagerV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match event {
            wayland_protocols::ext::workspace::v1::client::ext_workspace_manager_v1::Event::WorkspaceGroup { workspace_group: _ } => todo!(),
            wayland_protocols::ext::workspace::v1::client::ext_workspace_manager_v1::Event::Workspace { workspace: _ } => todo!(),
            wayland_protocols::ext::workspace::v1::client::ext_workspace_manager_v1::Event::Done => todo!(),
            wayland_protocols::ext::workspace::v1::client::ext_workspace_manager_v1::Event::Finished => todo!(),
            _ => todo!(),
        }
    }
}

// The main function of our program
fn main() {
    let conn = Connection::connect_to_env().unwrap();

    let display = conn.display();

    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    let _registry = display.get_registry(&qh, ());

    println!("Advertised globals:");

    loop {
        event_queue.blocking_dispatch(&mut AppData {}).unwrap();
    }
}
