use crate::config::de_duration;

use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Connection {
    #[serde(rename = "serial")]
    Serial(SocketConnection),
    #[serde(rename = "socket")]
    Socket(SocketConnection),
    #[serde(rename = "rpmsg")]
    Rpmsg(RpmsgConnection),
}

fn default_timeout() -> Duration {
    Duration::from_millis(100)
}

#[derive(Debug, Deserialize)]
pub struct SocketConnection {
    pub path: String,
    pub baud: Option<u32>,
    #[serde(default = "default_timeout", deserialize_with = "de_duration")]
    pub timeout: Duration,
}

fn default_settle() -> Duration {
    Duration::from_millis(10)
}

fn default_rpmsg_timeout() -> Duration {
    Duration::from_millis(100)
}

#[derive(Debug, Deserialize)]
pub struct RpmsgConnection {
    #[serde(default = "default_rpmsg_ctrl_path")]
    pub ctrl_path: String,
    pub channel_name: String,
    #[serde(default = "default_remoteproc_path")]
    pub remoteproc_state_path: String,
    #[serde(default = "default_settle", deserialize_with = "de_duration")]
    pub settle: Duration,
    #[serde(default = "default_rpmsg_timeout", deserialize_with = "de_duration")]
    pub timeout: Duration,
}

fn default_rpmsg_ctrl_path() -> String {
    "/dev/rpmsg_ctrl-dsp_rproc@0".to_string()
}

fn default_remoteproc_path() -> String {
    "/sys/class/remoteproc/remoteproc0/state".to_string()
}
