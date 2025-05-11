use zbus::blocking::fdo::DBusProxy;

pub struct DBusWrapper {
    conn: zbus::blocking::Connection,
}

impl DBusWrapper {
    pub fn new() -> Result<Self, zbus::Error> {
        Ok(Self {
            conn: zbus::blocking::Connection::session()?,
        })
    }

    pub fn name_has_owner(&self, name: &str) -> Result<bool, zbus::Error> {
        Ok(DBusProxy::new(&self.conn)?.name_has_owner(zbus_names::BusName::try_from(name)?)?)
    }

    pub fn wait_for_name_owner(&self, name: &str) -> Result<(), zbus::Error> {
        let mut stream = DBusProxy::new(&self.conn)?.receive_name_owner_changed()?;

        while let Some(message) = stream.next() {
            if let Ok(args) = message.args() {
                if let zbus_names::BusName::WellKnown(dbus_name) = args.name {
                    if dbus_name == name {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}
