pub mod gpio;
pub mod rpmsg;
pub mod socket;

pub trait Stream: std::io::Read + std::io::Write {}
