use chrono::Local;

pub fn ts() -> String {
    Local::now().format("[%a %b %e %H:%M:%S]").to_string()
}

#[macro_export]
macro_rules! vb_log {
    ($prefix:expr, $($arg:tt)*) => {
        println!("{} {} {}", $crate::debug::ts(), $prefix, format!($($arg)*));
    };
}