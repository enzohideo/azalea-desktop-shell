use zbus::proxy;

/// Login
///
/// DBus interface responsible for shutdown, reboot, suspend, hibernate, etc.
///
/// See: https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.login1.html
#[proxy(
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1",
    interface = "org.freedesktop.login1.Manager"
)]
pub trait LoginManager {
    fn power_off(&self, interactive: bool) -> zbus::Result<()>;
    fn reboot(&self, interactive: bool) -> zbus::Result<()>;
    fn suspend(&self, interactive: bool) -> zbus::Result<()>;
    fn hibernate(&self, interactive: bool) -> zbus::Result<()>;
}
