pub static LOG_NAME: &str = "Azalea";
pub use glib;

#[macro_export]
macro_rules! error {
    ($format:literal, $($arg:expr),* $(,)?) => {{
        $crate::glib::g_log!($crate::LOG_NAME, $crate::glib::LogLevel::Error, $format, $($arg),*);
        panic!()
    }};
    ($format:literal $(,)?) => {{
        $crate::glib::g_log!($crate::LOG_NAME, $crate::glib::LogLevel::Error, $format);
        panic!()
    }};
    ($context:ty, $format:literal, $($arg:expr),* $(,)?) => {{
        $crate::error!("[{}] {}", std::any::type_name::<$context>(), format!($format, $($arg),*));
    }};
    ($context:ty, $format:literal $(,)?) => {{
        $crate::error!("[{}] {}", std::any::type_name::<$context>(), $format);
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
    ($context:ty, $format:literal, $($arg:expr),* $(,)?) => {{
        $crate::critical!("[{}] {}", std::any::type_name::<$context>(), format!($format, $($arg),*));
    }};
    ($context:ty, $format:literal $(,)?) => {{
        $crate::critical!("[{}] {}", std::any::type_name::<$context>(), $format);
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
    ($context:ty, $format:literal, $($arg:expr),* $(,)?) => {{
        $crate::warning!("[{}] {}", std::any::type_name::<$context>(), format!($format, $($arg),*));
    }};
    ($context:ty, $format:literal $(,)?) => {{
        $crate::warning!("[{}] {}", std::any::type_name::<$context>(), $format);
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
    ($context:ty, $format:literal, $($arg:expr),* $(,)?) => {{
        $crate::message!("[{}] {}", std::any::type_name::<$context>(), format!($format, $($arg),*));
    }};
    ($context:ty, $format:literal $(,)?) => {{
        $crate::message!("[{}] {}", std::any::type_name::<$context>(), $format);
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
    ($context:ty, $format:literal, $($arg:expr),* $(,)?) => {{
        $crate::info!("[{}] {}", std::any::type_name::<$context>(), format!($format, $($arg),*));
    }};
    ($context:ty, $format:literal $(,)?) => {{
        $crate::info!("[{}] {}", std::any::type_name::<$context>(), $format);
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
    ($context:ty, $format:literal, $($arg:expr),* $(,)?) => {{
        $crate::debug!("[{}] {}", std::any::type_name::<$context>(), format!($format, $($arg),*));
    }};
    ($context:ty, $format:literal $(,)?) => {{
        $crate::debug!("[{}] {}", std::any::type_name::<$context>(), $format);
    }};
}
