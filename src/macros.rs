// Macros.

#[macro_export]
macro_rules! debugf {
    ($($arg:tt)*) => (if cfg!(debug_assertions) { print!($($arg)*) })
}
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => (if cfg!(debug_assertions) { println!($($arg)*) })
}
