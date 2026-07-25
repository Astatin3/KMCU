pub type Error = String;

pub type Res<T> = core::result::Result<T, Error>;

macro_rules! Err {
    ($fmt:expr, $($arg:tt)*) => {
        $crate::Error::msg($crate::__private::format!($fmt, $($arg)*))
    };
}
