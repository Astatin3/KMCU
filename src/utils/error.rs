pub type Res<T> = core::result::Result<T, Error>;

#[cfg(feature = "log")]
pub type Error = alloc::string::String;

#[cfg(feature = "log")]
macro_rules! err {
    ($fmt:expr) => {
        alloc::format!($fmt)
    };
    ($fmt:expr, $($arg:tt)*) => {
        alloc::format!($fmt, $($arg)*)
    };
}

#[cfg(not(feature = "log"))]
pub type Error = ();

#[cfg(not(feature = "log"))]
macro_rules! err {
    ($msg:expr) => {
        ()
    };
    ($fmt:expr, $($arg:tt)*) => {
        ()
    };
}
