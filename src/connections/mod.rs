pub mod rpmsg;
pub mod serial;
pub mod socket;

use std::time::Duration;

/// Combined Read + Write trait for trait objects.
pub trait Stream: std::io::Read + std::io::Write {
    /// Reconnect the underlying transport (e.g. after DSP restart).
    fn reconnect(&mut self) -> anyhow::Result<()> {
        anyhow::bail!("reconnect not supported for this stream type")
    }

    /// Per-frame read timeout.
    fn read_timeout(&self) -> Duration {
        Duration::from_secs(10)
    }
}
