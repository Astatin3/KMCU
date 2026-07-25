pub type Res<T> = core::result::Result<T, Error>;

#[cfg(feature = "log")]
pub type Error = String;

#[cfg(feature = "log")]
macro_rules! err {
    ($msg:expr) => {
        String::from($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        format!($fmt, $($arg)*)
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
