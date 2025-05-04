pub static LOG_NAME: &str = "Azalea";

#[macro_export]
macro_rules! error {
    ($format:literal, $($arg:expr),* $(,)?) => {{
        gtk::glib::g_log!($crate::log::LOG_NAME, gtk::glib::LogLevel::Error, $format, $($arg),*);
    }};
    ($format:literal $(,)?) => {{
        gtk::glib::g_log!($crate::log::LOG_NAME, gtk::glib::LogLevel::Error, $format);
    }};
}

#[macro_export]
macro_rules! info {
    ($format:literal, $($arg:expr),* $(,)?) => {{
        gtk::glib::g_log!($crate::log::LOG_NAME, gtk::glib::LogLevel::Info, $format, $($arg),*);
    }};
    ($format:literal $(,)?) => {{
        gtk::glib::g_log!($crate::log::LOG_NAME, gtk::glib::LogLevel::Info, $format);
    }};
}

#[macro_export]
macro_rules! warning {
    ($format:literal, $($arg:expr),* $(,)?) => {{
        gtk::glib::g_log!($crate::log::LOG_NAME, gtk::glib::LogLevel::Warning, $format, $($arg),*);
    }};
    ($format:literal $(,)?) => {{
        gtk::glib::g_log!($crate::log::LOG_NAME, gtk::glib::LogLevel::Warning, $format);
    }};
}

#[macro_export]
macro_rules! message {
    ($format:literal, $($arg:expr),* $(,)?) => {{
        gtk::glib::g_log!($crate::log::LOG_NAME, gtk::glib::LogLevel::Message, $format, $($arg),*);
    }};
    ($format:literal $(,)?) => {{
        gtk::glib::g_log!($crate::log::LOG_NAME, gtk::glib::LogLevel::Message, $format);
    }};
}

pub use error;
pub use info;
pub use message;
pub use warning;
