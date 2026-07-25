#[cfg(feature = "platform_linux")]
mod linux {
    mod clock;
    mod gpio;
    mod rpmsg;
    mod socket;

    pub use clock::{now, sleep};
    pub use gpio::GPIO;
    pub use rpmsg::RpmsgEndpoint;
    pub use socket::Socket;
}

mod dummy {
    mod clock;
    mod gpio;
    mod rpmsg;
    mod socket;

    pub use clock::{now, sleep};
    pub use gpio::GPIO;
    pub use rpmsg::RpmsgEndpoint;
    pub use socket::Socket;
}

#[cfg(feature = "platform_linux")]
pub use linux::*;

#[cfg(not(feature = "platform_linux"))]
pub use dummy::*;
