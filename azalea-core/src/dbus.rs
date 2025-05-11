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
}
