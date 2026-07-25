use crate::units;
use alloc::string::{String, ToString};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Connection {
    #[serde(rename = "serial")]
    Serial(SerialConnection),
    #[serde(rename = "socket")]
    Socket(SocketConnection),
    #[serde(rename = "rpmsg")]
    Rpmsg(RpmsgConnection),
}

fn default_timeout() -> units::LongTime {
    units::LongTime::new::<units::long_millisecond>(100)
}

#[derive(Debug, Deserialize)]
pub struct SerialConnection {
    pub path: String,
    pub baud: u32,
    #[serde(default = "default_timeout")]
    pub timeout: units::LongTime,
}

#[derive(Debug, Deserialize)]
pub struct SocketConnection {
    pub path: String,
    #[serde(default = "default_timeout")]
    pub timeout: units::LongTime,
}

fn default_settle() -> units::LongTime {
    units::LongTime::new::<units::long_millisecond>(10)
}

fn default_rpmsg_timeout() -> units::LongTime {
    units::LongTime::new::<units::long_millisecond>(100)
}

#[derive(Debug, Deserialize)]
pub struct RpmsgConnection {
    #[serde(default = "default_rpmsg_ctrl_path")]
    pub ctrl_path: String,
    pub channel_name: String,
    #[serde(default = "default_remoteproc_path")]
    pub remoteproc_state_path: String,

    #[serde(default = "default_settle")]
    pub settle: units::LongTime,
    #[serde(default = "default_rpmsg_timeout")]
    pub timeout: units::LongTime,
}

fn default_rpmsg_ctrl_path() -> String {
    "/dev/rpmsg_ctrl-dsp_rproc@0".to_string()
}

fn default_remoteproc_path() -> String {
    "/sys/class/remoteproc/remoteproc0/state".to_string()
}
