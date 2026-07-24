pub mod rpmsg;
pub mod serial;
pub mod socket;

pub trait Stream: std::io::Read + std::io::Write {}
