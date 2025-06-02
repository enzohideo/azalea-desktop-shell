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

pub fn error<Obj>(text: &str) -> ! {
    error!("[{}] {}", std::any::type_name::<Obj>(), text);
    loop {
        panic!()
    }
}

pub fn critical<Obj>(text: &str) {
    critical!("[{}] {}", std::any::type_name::<Obj>(), text);
}

pub fn warning<Obj>(text: &str) {
    warning!("[{}] {}", std::any::type_name::<Obj>(), text);
}

pub fn message<Obj>(text: &str) {
    message!("[{}] {}", std::any::type_name::<Obj>(), text);
}

pub fn info<Obj>(text: &str) {
    info!("[{}] {}", std::any::type_name::<Obj>(), text);
}

pub fn debug<Obj>(text: &str) {
    debug!("[{}] {}", std::any::type_name::<Obj>(), text);
}
