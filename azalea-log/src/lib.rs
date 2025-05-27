pub static LOG_NAME: &str = "Azalea";
pub use glib;

#[macro_export]
macro_rules! error {
    ($format:literal, $($arg:expr),* $(,)?) => {{
        $crate::glib::g_log!($crate::LOG_NAME, $crate::glib::LogLevel::Error, $format, $($arg),*);
    }};
    ($format:literal $(,)?) => {{
        $crate::glib::g_log!($crate::LOG_NAME, $crate::glib::LogLevel::Error, $format);
    }};
}

#[macro_export]
macro_rules! critical {
    ($format:literal, $($arg:expr),* $(,)?) => {{
        $crate::glib::g_log!($crate::LOG_NAME, $crate::glib::LogLevel::Critical, $format, $($arg),*);
    }};
    ($format:literal $(,)?) => {{
        $crate::glib::g_log!($crate::LOG_NAME, $crate::glib::LogLevel::Critical, $format);
    }};
}

#[macro_export]
macro_rules! warning {
    ($format:literal, $($arg:expr),* $(,)?) => {{
        $crate::glib::g_log!($crate::LOG_NAME, $crate::glib::LogLevel::Warning, $format, $($arg),*);
    }};
    ($format:literal $(,)?) => {{
        $crate::glib::g_log!($crate::LOG_NAME, $crate::glib::LogLevel::Warning, $format);
    }};
}

#[macro_export]
macro_rules! message {
    ($format:literal, $($arg:expr),* $(,)?) => {{
        $crate::glib::g_log!($crate::LOG_NAME, $crate::glib::LogLevel::Message, $format, $($arg),*);
    }};
    ($format:literal $(,)?) => {{
        $crate::glib::g_log!($crate::LOG_NAME, $crate::glib::LogLevel::Message, $format);
    }};
}

#[macro_export]
macro_rules! info {
    ($format:literal, $($arg:expr),* $(,)?) => {{
        $crate::glib::g_log!($crate::LOG_NAME, $crate::glib::LogLevel::Info, $format, $($arg),*);
    }};
    ($format:literal $(,)?) => {{
        $crate::glib::g_log!($crate::LOG_NAME, $crate::glib::LogLevel::Info, $format);
    }};
}

#[macro_export]
macro_rules! debug {
    ($format:literal, $($arg:expr),* $(,)?) => {{
        $crate::glib::g_log!($crate::LOG_NAME, $crate::glib::LogLevel::Debug, $format, $($arg),*);
    }};
    ($format:literal $(,)?) => {{
        $crate::glib::g_log!($crate::LOG_NAME, $crate::glib::LogLevel::Debug, $format);
    }};
}
